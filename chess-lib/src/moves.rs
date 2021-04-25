use crate::board::{directions, is_in_bounds, rank, Coords, Direction, Line};
use crate::types::{Board, Colour, Coordinate, GameState, Move, Piece, Square};

pub fn legal_moves(state: &GameState) -> Vec<Move> {
    let colour = state.active_colour;

    let side = match colour {
        Colour::White => &state.white,
        Colour::Black => &state.black,
    };

    let king_coord = side.king_coord;

    // Calculate restrictions on allowed moves based on checks and pins.
    let attacks = attacks_on_square(&state.board, king_coord, colour);

    // If there are checks, this bitfield will contain the squares which stop ALL checks.
    // i.e. The only legal moves are moves that move to these squares, or king moves.
    let mut allowed_non_king_moves: u128 = 0xFFFFFFFF_FFFFFFFF_FFFFFFFF_FFFFFFFF;

    // This is a map from a pinned piece, to the coordinates it can move to without
    // breaking the pin.
    // i.e. if a piece is in this map, it can ONLY move to the listed squares.
    let mut pins: [Option<u128>; 120] = [None; 120];
    attacks.into_iter().for_each(|atk| {
        match atk {
            Attack::Check(bs) => {
                allowed_non_king_moves &= bs;
            },
            Attack::Pin(c, bs) => {
                pins[c as usize] = Some(bs);
            },
        }
    });

    // Now get all moves disregarding restrictions.
    let piece_coords = Coords::new().filter(|c| match state.board[*c as usize] {
        Square::Occupied(col, _) => col == colour,
        _ => false,
    });

    let moves_without_restrictions = piece_coords.flat_map(|coord| {
        piece_movement(&state.board, coord, state.en_passant)
            .map(move |tgt| Move::Normal(coord, tgt))
    });

    // Move only allowed if it blocks all checks, or is king move.
    let moves_respecting_checks = moves_without_restrictions.filter(|m| {
        match m {
            Move::Normal(src, tgt) => {
                // Special case allow en passant past this check since we'll check it manually
                // later.
                match state.board[*src as usize] {
                    Square::Occupied(_, Piece::Pawn) => {
                        if state.en_passant.map(|ep| ep == *tgt).unwrap_or(false) {
                            return true;
                        }
                    },
                    _ => (),
                };

                allowed_non_king_moves & (1 << *tgt) != 0 || *src == king_coord
            },
            _ => true,
        }
    });

    // Move only allowed if it doesn't break a pin.
    let moves_respecting_pins = moves_respecting_checks.filter(|m| {
        match m {
            Move::Normal(src, tgt) => {
                match pins[*src as usize] {
                    Some(allowed_moves) => allowed_moves & (1 << *tgt) != 0,
                    None => true,
                }
            },
            _ => true,
        }
    });

    // Cloning the whole board probably not the most efficient.
    let mut board_without_king = state.board.clone();
    board_without_king[king_coord as usize] = Square::Empty;

    // Remove king moves which would put the king in check.
    let moves_without_suicidal_king = moves_respecting_pins.filter(|m| {
        match m {
            Move::Normal(src, tgt) => {
                if *src == king_coord {
                    !square_under_attack(&board_without_king, *tgt, colour)
                } else {
                    true
                }
            },
            _ => true,
        }
    });

    // This is as far as we can reasonably go using Iterators.
    // Collect now and do the rest in a loop.
    let mut moves: Vec<Move> = Vec::with_capacity(256);
    for m in moves_without_suicidal_king {
        match m {
            Move::Normal(src, tgt) => {
                match state.board[src as usize] {
                    Square::Occupied(_, Piece::Pawn) => {
                        // Expand pawn moves to last rank.
                        // Don't have to check colours since pawns can't move backwards.
                        if rank(tgt) == 7 || rank(tgt) == 0 {
                            moves.push(Move::Promotion(src, tgt, Piece::Queen));
                            moves.push(Move::Promotion(src, tgt, Piece::Rook));
                            moves.push(Move::Promotion(src, tgt, Piece::Bishop));
                            moves.push(Move::Promotion(src, tgt, Piece::Knight));
                        } else if state.en_passant.map(|ep| ep == tgt).unwrap_or(false) {
                            // Remove en-passant if it would leave the king in check.
                            // Can't think of a better way to do this than just evaluating the new
                            // board for checks.
                            let mut new_board = state.board.clone();
                            let taken_coord = tgt.wrapping_add(if colour == Colour::White { directions::DOWN } else { directions::UP });
                            new_board[tgt as usize] = new_board[src as usize];
                            new_board[src as usize] = Square::Empty;
                            new_board[taken_coord as usize] = Square::Empty;
                            if !square_under_attack(&new_board, side.king_coord, colour) {
                                moves.push(m);
                            }
                        } else {
                            moves.push(m);
                        }
                    },
                    _ => moves.push(m),
                }
            },
            _ => panic!("Should only have normal moves at this stage"),
        }
    }


    // Add castling if legal.
    let home_rank = match colour {
        Colour::White => 0x00,
        Colour::Black => 0x07,
    };

    if side.can_castle_queenside {
        if !(
            board_without_king[0x10 | home_rank as usize] != Square::Empty ||
            board_without_king[0x20 | home_rank as usize] != Square::Empty ||
            board_without_king[0x30 | home_rank as usize] != Square::Empty ||
            square_under_attack(&board_without_king, 0x20 | home_rank, colour) ||
            square_under_attack(&board_without_king, 0x30 | home_rank, colour) ||
            square_under_attack(&board_without_king, 0x40 | home_rank, colour)
        ) {
            moves.push(Move::LongCastle);
        }
    }

    if side.can_castle_kingside {
        if !(
            board_without_king[0x50 | home_rank as usize] != Square::Empty ||
            board_without_king[0x60 | home_rank as usize] != Square::Empty ||
            square_under_attack(&board_without_king, 0x40 | home_rank, colour) ||
            square_under_attack(&board_without_king, 0x50 | home_rank, colour) ||
            square_under_attack(&board_without_king, 0x60 | home_rank, colour)
        ) {
            moves.push(Move::Castle);
        }
    }

    moves
}

#[derive(Debug)]
enum Attack {
    Check(u128),
    Pin(Coordinate, u128),
}

pub fn square_under_attack(board: &Board, coord: Coordinate, colour: Colour) -> bool {
    attacks_on_square(board, coord, colour)
        .iter()
        .any(|atk| match atk {
            Attack::Check(_) => true,
            _ => false,
        })
}

fn attacks_on_square(board: &Board, coord: Coordinate, colour: Colour) -> Vec<Attack> {
    let mut attacks: Vec<Attack> = Vec::with_capacity(8);

    // Regular pieces.
    directions::ALL.as_ref().into_iter().for_each(|dir| {
        let mut dist = 0;
        let mut blocks = 0u128;
        let mut pin: Option<Coordinate> = None;

        // Find pins and straight line checks.
        for c in Line::new(coord, *dir) {
            dist += 1;
            blocks |= 1 << c;
            match board[c as usize] {
                Square::Empty => continue,
                Square::Occupied(col, piece) => {
                    if col == colour {
                        if pin.is_some() {
                            // Two blocking pieces, no attack.
                            break;
                        } else {
                            pin = Some(c);
                        }
                    } else {
                        if piece_attacks_in_direction(col, piece, directions::reverse(*dir)) && piece_attacks_at_distance(piece, dist) {
                            // Attack is real.
                            match pin {
                                None => attacks.push(Attack::Check(blocks)),
                                Some(p) => attacks.push(Attack::Pin(p, blocks)),
                            }
                            break;
                        } else {
                            // Blocked by opposing non-attacking piece.
                            break;
                        }
                    }
                },
            }
        }
    });

    // Check knights.
    directions::KNIGHT.as_ref().into_iter().for_each(|dir| {
        let knight_coord = coord.wrapping_add(*dir);
        if is_in_bounds(knight_coord) {
            match board[knight_coord as usize] {
                Square::Occupied(col, Piece::Knight) => {
                    if col != colour {
                        // Taking the knight is the only way to block the check.
                        attacks.push(Attack::Check(1 << knight_coord))
                    }
                },
                _ => (),
            }
        }
    });

    attacks
}

fn piece_attacks_at_distance(piece: Piece, dist: u8) -> bool {
    match piece {
        Piece::King => dist == 1,
        Piece::Pawn => dist == 1,
        Piece::Knight => false,
        _ => true,
    }
}

fn piece_attacks_in_direction(colour: Colour, piece: Piece, dir: Direction) -> bool {
    match piece {
        Piece::King => true,
        Piece::Queen => true,
        Piece::Rook => directions::is_straight(dir),
        Piece::Bishop => directions::is_diagonal(dir),
        Piece::Knight => false,
        Piece::Pawn => {
            match colour {
                Colour::White => dir == directions::UP_LEFT || dir == directions::UP_RIGHT,
                Colour::Black => dir == directions::DOWN_LEFT || dir == directions::DOWN_RIGHT,
            }
        },
    }
}

enum PieceMoves<'a> {
    King(LineMoves<'a>),
    Queen(LineMoves<'a>),
    Rook(LineMoves<'a>),
    Bishop(LineMoves<'a>),
    Knight(LineMoves<'a>),
    Pawn(PawnMoves<'a>),
}

impl <'a> Iterator for PieceMoves<'a> {
    type Item = Coordinate;

    fn next(&mut self) -> Option<Coordinate> {
        match self {
            PieceMoves::King(iter) => iter.next(),
            PieceMoves::Queen(iter) => iter.next(),
            PieceMoves::Rook(iter) => iter.next(),
            PieceMoves::Bishop(iter) => iter.next(),
            PieceMoves::Knight(iter) => iter.next(),
            PieceMoves::Pawn(iter) => iter.next(),
        }
    }
}

struct LineMoves<'a> {
    board: &'a Board,
    coord: Coordinate,
    colour: Colour,
    range: usize,
    dirs: std::slice::Iter<'a, Direction>,

    cur_dir: Option<UntilBlocked<'a, std::iter::Take<Line>>>,
}

impl <'a> LineMoves<'a> {
    fn new(
        board: &'a Board,
        coord: Coordinate,
        colour: Colour,
        dirs: std::slice::Iter<'a, Direction>,
        range: usize,
    ) -> LineMoves<'a> {
        LineMoves{
            board,
            coord,
            colour,
            range,
            dirs: dirs,
            cur_dir: None,
        }
    }
}

impl <'a> Iterator for LineMoves<'a> {
    type Item = Coordinate;

    fn next(&mut self) -> Option<Coordinate> {
        loop {
            let next_coord = self.cur_dir.as_mut().and_then(|iter| iter.next());

            match next_coord {
                Some(c) => return Some(c),
                None => {
                    match self.dirs.next() {
                        Some(dir) => {
                            self.cur_dir = Some(moves_in_line(self.board, self.colour, Line::new(self.coord, *dir), self.range).into_iter());
                        },
                        None => return None,
                    }
                },
            }
        }
    }
}

struct PawnMoves<'a> {
    board: &'a Board,
    coord: Coordinate,
    colour: Colour,
    en_passant: Option<Coordinate>,

    step: u8,
}

impl <'a> PawnMoves<'a> {
    fn new(board: &'a Board, coord: Coordinate, colour: Colour, en_passant: Option<Coordinate>) -> PawnMoves {
        PawnMoves{
            board,
            coord,
            colour,
            en_passant,
            step: 0,
        }
    }

    fn can_capture(&self, tgt: Coordinate) -> bool {
        let regular_capture = can_capture(self.board, self.colour, tgt);
        let en_passant = self.en_passant.map(|ep| ep == tgt).unwrap_or(false);
        regular_capture || en_passant
    }
}

impl <'a> Iterator for PawnMoves<'a> {
    type Item = Coordinate;

    fn next(&mut self) -> Option<Coordinate> {
        let (fwd, d1, d2, home_rank) = match self.colour {
            Colour::White => (directions::UP, directions::UP_LEFT, directions::UP_RIGHT, 0x01),
            Colour::Black => (directions::DOWN, directions::DOWN_LEFT, directions::DOWN_RIGHT, 0x06),
        };

        loop {
            match self.step {
                0 => {
                    // Single forward step.
                    let tgt = self.coord.wrapping_add(fwd);
                    if is_in_bounds(tgt) && self.board[tgt as usize] == Square::Empty {
                        self.step += 1;
                        return Some(tgt);
                    } else {
                        // If the square in front was blocked, don't bother checking the square 2
                        // in front.
                        self.step += 2;
                    }
                },
                1 => {
                    // Double forward step.
                    // Note we skip this entirely if the square in front is blocked.
                    // so no need to check for a piece in between.
                    let tgt = self.coord.wrapping_add(fwd.wrapping_mul(2));
                    self.step += 1;
                    if rank(self.coord) == home_rank && is_in_bounds(tgt) && self.board[tgt as usize] == Square::Empty {
                        return Some(tgt);
                    }
                },
                2 => {
                    // Diagonal capture 1.
                    let tgt = self.coord.wrapping_add(d1);
                    self.step += 1;
                    if is_in_bounds(tgt) && self.can_capture(tgt) {
                        return Some(tgt);
                    }
                },
                3 => {
                    // Diagonal capture 1.
                    let tgt = self.coord.wrapping_add(d2);
                    self.step += 1;
                    if is_in_bounds(tgt) && self.can_capture(tgt) {
                        return Some(tgt);
                    }
                },
                4 => return None,
                _ => panic!("Invalid iterator state: step={}", self.step),
            }
        }
    }
}

fn piece_movement(board: &Board, coord: Coordinate, en_passant: Option<Coordinate>) -> PieceMoves {
    let (colour, piece) = match board[coord as usize] {
        Square::Occupied(p, c) => (p, c),
        Square::Empty => panic!("No piece on square {}", coord),
    };

    match piece {
        Piece::King => {
            PieceMoves::King(LineMoves::new(board, coord, colour, directions::ALL.as_ref().into_iter(), 1))
        },
        Piece::Queen => {
            PieceMoves::Queen(LineMoves::new(board, coord, colour, directions::ALL.as_ref().into_iter(), 8))
        },
        Piece::Rook => {
            PieceMoves::Rook(LineMoves::new(board, coord, colour, directions::STRAIGHTS.as_ref().into_iter(), 8))
        },
        Piece::Bishop => {
            PieceMoves::Bishop(LineMoves::new(board, coord, colour, directions::DIAGONALS.as_ref().into_iter(), 8))
        },
        Piece::Knight => {
            PieceMoves::Knight(LineMoves::new(board, coord, colour, directions::KNIGHT.as_ref().into_iter(), 1))
        },
        Piece::Pawn => {
            PieceMoves::Pawn(PawnMoves::new(&board, coord, colour, en_passant))
        },
    }
}

fn moves_in_line<'a>(board: &'a Board, colour: Colour, line: Line, limit: usize) -> UntilBlocked<'a, std::iter::Take<Line>> {
    UntilBlocked::new(board, colour, true, line.take(limit))
}

fn can_capture(board: &Board, colour: Colour, coord: Coordinate) -> bool {
        match board[coord as usize] {
            Square::Empty => false,
            Square::Occupied(c, _) => c != colour,
        }
}

struct UntilBlocked<'a, I> {
    coords: I,
    board: &'a Board,
    colour: Colour,
    blocked: bool,
    allow_captures: bool,
}

impl <'a, I : Iterator<Item=Coordinate>> UntilBlocked<'a, I> {
    fn new(board: &'a Board, colour: Colour, allow_captures: bool, coords: I) -> UntilBlocked<I> {
        UntilBlocked{
            coords,
            board,
            colour,
            blocked: false,
            allow_captures,
        }
    }
}

impl <'a, I : Iterator<Item=Coordinate>> Iterator for UntilBlocked<'a, I> {
    type Item = Coordinate;

    fn next(&mut self) -> Option<Coordinate> {
        if self.blocked {
            return None;
        }

        let nxt = match self.coords.next() {
            Some(c) => c,
            None => return None,
        };

        match self.board[nxt as usize] {
            Square::Empty => Some(nxt),
            Square::Occupied(c, _) => {
                self.blocked = true;
                if c != self.colour && self.allow_captures {
                    Some(nxt)
                } else {
                    None
                }
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use crate::board::{empty_board};
    use crate::moves::*;
    use crate::types::IntoCoord;

    macro_rules! test_movement {
        [ $name:ident: Given $( a $colour:ident $piece:ident on $coord:expr ),+, the piece on $src:expr, can move to $( $tgt:expr ),+ ] => {
            #[test]
            fn $name() {
                let mut board = empty_board();
                $(
                    board[$coord.into_coord() as usize] = Square::Occupied(Colour::$colour, Piece::$piece);
                )+
                let mut expected: Vec<Coordinate> = Vec::new();
                $(
                    expected.push($tgt.into_coord());
                )+
                let moves = piece_movement(&board, $src.into_coord(), None);
                let moves_set: HashSet<Coordinate> = moves.collect();
                let expected_set: HashSet<Coordinate> = expected.into_iter().collect();
                assert_eq!(moves_set, expected_set);
            }
        };
        [ $name:ident: Given $( a $colour:ident $piece:ident on $coord:expr ),+, the piece on $src:expr, cannot move ] => {
            #[test]
            fn $name() {
                let mut board = empty_board();
                $(
                    board[$coord.into_coord() as usize] = Square::Occupied(Colour::$colour, Piece::$piece);
                )+
                let moves = piece_movement(&board, $src.into_coord(), None);
                assert_eq!(moves.collect::<Vec<Coordinate>>(), Vec::new());
            }
        };
    }

    test_movement![ white_pawn_moves_up:
        Given a White Pawn on "d4",
        the piece on "d4",
        can move to "d5"
    ];

    test_movement![ black_pawn_moves_down:
        Given a Black Pawn on "d4",
        the piece on "d4",
        can move to "d3"
    ];

    test_movement![ pawn_gets_blocked:
        Given a White Pawn on "d4", a Black Pawn on "d5",
        the piece on "d4",
        cannot move
    ];

    test_movement![ pawn_can_capture_opposite_colour:
        Given a White Pawn on "d4", a Black Knight on "e5",
        the piece on "d4",
        can move to "d5", "e5"
    ];

    test_movement![ pawn_cannot_capture_same_colour:
        Given a White Pawn on "d4", a White Knight on "e5",
        the piece on "d4",
        can move to "d5"
    ];

    test_movement![ black_pawn_can_capture_downwards:
        Given a Black Pawn on "d4", a White Knight on "c3",
        the piece on "d4",
        can move to "c3", "d3"
    ];

    test_movement![ king_unobstructed:
        Given a White King on "d4",
        the piece on "d4",
        can move to "e3", "e4", "e5", "d3", "d5", "c3", "c4", "c5"
    ];

    test_movement![ king_in_corner:
        Given a White King on "h8",
        the piece on "h8",
        can move to "h7", "g7", "g8"
    ];

    test_movement![ knight_unobstructed:
        Given a Black Knight on "d4",
        the piece on "d4",
        can move to "b5", "c6", "e6", "f5", "f3", "e2", "c2", "b3"
    ];

    test_movement![ knight_in_corner:
        Given a White Knight on "a1",
        the piece on "a1",
        can move to "b3", "c2"
    ];

    test_movement![ knight_blocked:
        Given a White Knight on "a1", a White Queen on "c2",
        the piece on "a1",
        can move to "b3"
    ];

    test_movement![ knight_capture:
        Given a Black Knight on "a1", a White Queen on "c2",
        the piece on "a1",
        can move to "b3", "c2"
    ];

    test_movement![ rook_unobstructed:
        Given a White Rook on "d4",
        the piece on "d4",
        can move to "a4", "b4", "c4", "e4", "f4", "g4", "h4", "d1", "d2", "d3", "d5", "d6", "d7", "d8"
    ];

    test_movement![ rook_obstructed:
        Given a White Rook on "d4", a White Queen on "d2", a Black Queen on "f4",
        the piece on "d4",
        can move to "a4", "b4", "c4", "e4", "f4", "d3", "d5", "d6", "d7", "d8"
    ];

    test_movement![ bishop_unobstructed:
        Given a Black Bishop on "d4",
        the piece on "d4",
        can move to "a1", "b2", "c3", "e5", "f6", "g7", "h8", "a7", "b6", "c5", "e3", "f2", "g1"
    ];

    test_movement![ bishop_obstructed:
        Given a Black Bishop on "d4", a Black Knight on "c3", a White Rook on "c5",
        the piece on "d4",
        can move to "e5", "f6", "g7", "h8", "c5", "e3", "f2", "g1"
    ];
}
