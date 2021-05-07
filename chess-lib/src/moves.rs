use crate::board::{directions, is_in_bounds, rank, Direction, Line};
use crate::magic::MagicBitBoards;
use crate::types::{BitBoard, BitCoord, Board, Colour, Coordinate, GameState, IntoCoord, Move, Piece, Pieces, Square};

pub fn legal_moves(state: &GameState, mbb: &MagicBitBoards) -> Vec<Move> {
    let colour = state.active_colour;

    let (side, other_side) = match colour {
        Colour::White => (&state.white, &state.black),
        Colour::Black => (&state.black, &state.white),
    };

    let king_coord = BitCoord::from(Into::<u64>::into(side.pieces.king)).into_coord();

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
    let moves_without_restrictions = side.pieces.all()
        .iter()
        .flat_map(|src| {
            magic_piece_movement(&side.pieces, &other_side.pieces, src, colour, state.en_passant.map(|c| c.into()), mbb).iter()
                .map(move |tgt| Move::Normal(src.into_coord(), tgt.into_coord()))
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
                            if !square_under_attack(&new_board, king_coord, colour) {
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

fn magic_piece_movement(active_pieces: &Pieces, other_pieces: &Pieces, coord: BitCoord, colour: Colour, en_passant: Option<BitCoord>, mbb: &MagicBitBoards) -> BitBoard {
    let active_occupancy = active_pieces.all();
    let other_occupancy = other_pieces.all();
    let occupancy = active_occupancy | other_occupancy;

    let piece = match active_pieces.get_piece(coord) {
        Some(p) => p,
        None => return BitBoard::EMPTY,
    };

    let moves = match piece {
        Piece::Queen => {
            mbb.rook(coord).lookup(occupancy) | mbb.bishop(coord).lookup(occupancy)
        },
        Piece::Rook => {
            mbb.rook(coord).lookup(occupancy)
        },
        Piece::Bishop => {
            mbb.bishop(coord).lookup(occupancy)
        },
        Piece::Knight => {
            mbb.knight(coord)
        },
        Piece::King => {
            mbb.king(coord)
        },
        Piece::Pawn => {
            magic_pawn_moves(active_occupancy, other_occupancy, coord, colour, en_passant)
        },
    };

    // Remove moves that capture our own pieces.
    moves & (!active_pieces.all())
}

fn magic_pawn_moves(active_occupancy: BitBoard, other_occupancy: BitBoard, coord: BitCoord, colour: Colour, en_passant: Option<BitCoord>) -> BitBoard {
    let occupancy = active_occupancy | other_occupancy;
    match colour {
        Colour::White => {
            let home_row = BitBoard(0x00_00_00_00_00_00_FF_00);
            let mut moves = BitBoard::EMPTY;
            moves = moves | (coord << 8);
            if (home_row & coord != BitBoard::EMPTY) && (moves & occupancy == BitBoard::EMPTY) {
                moves = moves | (coord << 16);
            }
            moves = moves & (!occupancy);

            let tgts = other_occupancy | en_passant.unwrap_or(BitCoord(0));
            let mut atks = BitBoard::EMPTY;
            let file = (63 - coord.0.trailing_zeros()) % 8;
            if file != 0 {
                atks = atks | (coord << 9);
            }
            if file != 7 {
                atks = atks | (coord << 7);
            }

            moves | (atks & tgts)
        },
        Colour::Black => {
            let home_row = BitBoard(0x00_FF_00_00_00_00_00_00);
            let mut moves = BitBoard::EMPTY;
            moves = moves | (coord >> 8);
            if (home_row & coord != BitBoard::EMPTY) && (moves & occupancy == BitBoard::EMPTY) {
                moves = moves | (coord >> 16);
            }

            moves = moves & (!occupancy);

            let tgts = other_occupancy | en_passant.unwrap_or(BitCoord(0));
            let mut atks = BitBoard::EMPTY;
            let file = (63 - coord.0.trailing_zeros()) % 8;
            if file != 0 {
                atks = atks | (coord >>  7);
            }
            if file != 7 {
                atks = atks | (coord >> 9);
            }

            moves | (atks & tgts)
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
