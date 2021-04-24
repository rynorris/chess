use std::collections::{HashMap, HashSet};
use crate::board::{directions, is_in_bounds, rank, Direction, Line};
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

    // If there are checks, this set will contain the squares which stop ALL checks.
    // i.e. The only legal moves are moves that move to these squares, or king moves.
    let mut blocks: Option<HashSet<Coordinate>> = None;

    // This is a map from a pinned piece, to the coordinates it can move to without
    // breaking the pin.
    // i.e. if a piece is in this map, it can ONLY move to the listed squares.
    let mut pins: HashMap<Coordinate, HashSet<Coordinate>> = HashMap::new();
    attacks.into_iter().for_each(|atk| {
        match atk {
            Attack::Check(bs) => {
                if blocks.is_none() {
                    blocks = Some(bs);
                } else {
                    let all = blocks.take().unwrap();
                    blocks.replace(all.intersection(&bs).map(|c| *c).collect());
                }
            },
            Attack::Pin(c, bs) => {
                pins.insert(c, bs);
            },
        }
    });

    // Now get all moves disregarding restrictions.
    let piece_coords = &side.piece_coords;

    let moves_without_restrictions = piece_coords.iter().flat_map(|coord| {
        piece_movement(&state.board, *coord)
            .iter()
            .map(move |tgt| Move::Normal(*coord, *tgt))
            .collect::<Vec<Move>>()
    });

    // Move only allowed if it blocks all checks, or is king move.
    let moves_respecting_checks = moves_without_restrictions.filter(|m| {
        match blocks {
            None => true,
            Some(ref bs) => {
                    match m {
                        Move::Normal(src, tgt) => {
                            bs.contains(tgt) || *src == king_coord
                        },
                        _ => true,
                    }
            },
        }
    });

    // Move only allowed if it doesn't break a pin.
    let moves_respecting_pins = moves_respecting_checks.filter(|m| {
        match m {
            Move::Normal(src, tgt) => {
                match pins.get(src) {
                    Some(allowed) => allowed.contains(tgt),
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
            square_under_attack(&board_without_king, 0x20 & home_rank, colour) ||
            square_under_attack(&board_without_king, 0x30 & home_rank, colour) ||
            square_under_attack(&board_without_king, 0x40 & home_rank, colour) ||
            board_without_king[0x10 & home_rank as usize] != Square::Empty ||
            board_without_king[0x20 & home_rank as usize] != Square::Empty ||
            board_without_king[0x30 & home_rank as usize] != Square::Empty
        ) {
            moves.push(Move::LongCastle);
        }
    }

    if side.can_castle_kingside {
        if !(
            square_under_attack(&board_without_king, 0x40 & home_rank, colour) ||
            square_under_attack(&board_without_king, 0x50 & home_rank, colour) ||
            square_under_attack(&board_without_king, 0x60 & home_rank, colour) ||
            board_without_king[0x50 & home_rank as usize] != Square::Empty ||
            board_without_king[0x60 & home_rank as usize] != Square::Empty
        ) {
            moves.push(Move::Castle);
        }
    }

    moves
}

enum Attack {
    Check(HashSet<Coordinate>),
    Pin(Coordinate, HashSet<Coordinate>),
}

fn square_under_attack(board: &Board, coord: Coordinate, colour: Colour) -> bool {
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
        let mut blocks: HashSet<Coordinate> = HashSet::new();
        let mut pin: Option<Coordinate> = None;

        // Find pins and straight line checks.
        for c in Line::new(coord, *dir) {
            dist += 1;
            blocks.insert(c);
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
                        if piece_attacks_in_direction(colour, piece, directions::reverse(*dir)) && piece_attacks_at_distance(piece, dist) {
                            // Attack is real.
                            match pin {
                                None => attacks.push(Attack::Check(blocks)),
                                Some(p) => attacks.push(Attack::Pin(p, blocks)),
                            }
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
                        let mut blocks: HashSet<Coordinate> = HashSet::with_capacity(1);
                        blocks.insert(knight_coord);
                        attacks.push(Attack::Check(blocks))
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

pub fn piece_movement(board: &Board, coord: Coordinate) -> Vec<Coordinate> {
    let mut moves: Vec<Coordinate> = Vec::with_capacity(32);

    let (colour, piece) = match board[coord as usize] {
        Square::Occupied(p, c) => (p, c),
        Square::Empty => panic!("No piece on square {}", coord),
    };

    match piece {
        Piece::King => {
            directions::ALL.as_ref().into_iter().for_each(|dir| {
                moves.append(&mut moves_in_line(&board, colour, Line::new(coord, *dir), 1));
            });
        },
        Piece::Queen => {
            directions::ALL.as_ref().into_iter().for_each(|dir| {
                moves.append(&mut moves_in_line(&board, colour, Line::new(coord, *dir), 8));
            });
        },
        Piece::Rook => {
            directions::STRAIGHTS.as_ref().into_iter().for_each(|dir| {
                moves.append(&mut moves_in_line(&board, colour, Line::new(coord, *dir), 8));
            });
        },
        Piece::Bishop => {
            directions::DIAGONALS.as_ref().into_iter().for_each(|dir| {
                moves.append(&mut moves_in_line(&board, colour, Line::new(coord, *dir), 8));
            });
        },
        Piece::Knight => {
            moves.append(&mut knight_moves(&board, coord, colour));
        },
        Piece::Pawn => {
            moves.append(&mut pawn_moves(&board, coord, colour));
        },
    };

    moves
}

fn moves_in_line(board: &Board, colour: Colour, line: Line, limit: usize) -> Vec<Coordinate> {
    until_blocked(board, colour, &mut line.take(limit))
}

fn pawn_moves(board: &Board, coord: Coordinate, colour: Colour) -> Vec<Coordinate> {
    let (fwd, d1, d2) = match colour {
        Colour::White => (directions::UP, directions::UP_LEFT, directions::UP_RIGHT),
        Colour::Black => (directions::DOWN, directions::DOWN_LEFT, directions::DOWN_RIGHT),
    };

    let mut moves: Vec<Coordinate> = Vec::with_capacity(3);

    let initial_pawn_rank = match colour {
        Colour::White => 1,
        Colour::Black => 6,
    };

    let fwd_range = if rank(coord) == initial_pawn_rank { 2 } else { 1 };

    moves.extend(Line::new(coord, fwd).take(fwd_range).filter(|m| can_move_to(board, *m)));
    moves.extend(Line::new(coord, d1).take(1).filter(|m| can_capture(board, colour, *m)));
    moves.extend(Line::new(coord, d2).take(1).filter(|m| can_capture(board, colour, *m)));


    moves
}

fn knight_moves(board: &Board, coord: Coordinate, colour: Colour) -> Vec<Coordinate> {
    let mut moves: Vec<Coordinate> = Vec::with_capacity(8);

    moves.extend(
        directions::KNIGHT
            .as_ref()
            .into_iter()
            .map(|d| coord.wrapping_add(*d))
            .filter(|m| is_in_bounds(*m))
            .filter(|m| can_move_to(board, *m) || can_capture(board, colour, *m))
    );

    moves
}

fn can_move_to(board: &Board, coord: Coordinate) -> bool {
        match board[coord as usize] {
            Square::Empty => true,
            Square::Occupied(_, _) => false,
        }
}

fn can_capture(board: &Board, colour: Colour, coord: Coordinate) -> bool {
        match board[coord as usize] {
            Square::Empty => false,
            Square::Occupied(c, _) => c != colour,
        }
}

fn until_blocked(board: &Board, colour: Colour, moves_iter: &mut dyn Iterator<Item=Coordinate>) -> Vec<Coordinate> {
    let mut moves: Vec<Coordinate> = Vec::with_capacity(7);

    loop {
        match moves_iter.next() {
            Some(m) => {
                match board[m as usize] {
                    Square::Empty => moves.push(m),
                    Square::Occupied(c, _) => {
                        if c != colour {
                            moves.push(m);
                        }
                        break;
                    },
                };
            },
            None => break,
        }
    }

    moves
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
                let moves = piece_movement(&board, $src.into_coord());
                let moves_set: HashSet<_> = moves.iter().collect();
                let expected_set: HashSet<_> = expected.iter().collect();
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
                let moves = piece_movement(&board, $src.into_coord());
                assert_eq!(moves, Vec::new());
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

    #[test]
    fn perft_1() {
        let state = crate::fen::load_fen(crate::fen::STARTING_POSITION);
        let moves = legal_moves(&state);
        println!("{:?}", moves);
        assert_eq!(moves.len(), 20);
    }
}
