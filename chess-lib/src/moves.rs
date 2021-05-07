use crate::board::{rank};
use crate::magic::{Line, MagicBitBoards};
use crate::types::{BitBoard, BitCoord, Colour, Coordinate, GameState, IntoCoord, Move, Piece, Pieces, Square};

pub fn legal_moves(state: &GameState, mbb: &MagicBitBoards) -> Vec<Move> {
    let colour = state.active_colour;

    let (side, other_side) = match colour {
        Colour::White => (&state.white, &state.black),
        Colour::Black => (&state.black, &state.white),
    };

    // Calculate restrictions on allowed moves based on checks and pins.
    let occupancy = side.pieces.all() | other_side.pieces.all();
    let attacks = attacks_on_square(occupancy, &other_side.pieces, BitCoord(side.pieces.king.0), colour, mbb);

    // If there are checks, this bitfield will contain the squares which stop ALL checks.
    // i.e. The only legal moves are moves that move to these squares, or king moves.
    let mut allowed_non_king_moves = BitBoard(0xFF_FF_FF_FF_FF_FF_FF_FF);

    // This is a map from a pinned piece, to the coordinates it can move to without
    // breaking the pin.
    // i.e. if a piece is in this map, it can ONLY move to the listed squares.
    let mut pins: [Option<BitBoard>; 64] = [None; 64];
    let mut is_in_check = false;
    attacks.into_iter().for_each(|atk| {
        match atk {
            Attack::Check(bs) => {
                allowed_non_king_moves = allowed_non_king_moves & bs;
                is_in_check = true;
            },
            Attack::Pin(c, bs) => {
                pins[c.0.trailing_zeros() as usize] = Some(bs);
            },
        }
    });

    // Now get all moves disregarding restrictions.
    let mostly_legal_moves = side.pieces.all()
        .iter()
        .flat_map(|src| {
            let mut pseudo_legals = magic_piece_movement(&side.pieces, &other_side.pieces, src, colour, state.en_passant.map(|c| c.into()), mbb);
            
            // If there's en-passant, store it off and add it back in later.
            let ep_mask = if side.pieces.pawns & src != BitBoard::EMPTY {
                state.en_passant.map(|ep| pseudo_legals & ep).unwrap_or(BitBoard::EMPTY)
            } else {
                BitBoard::EMPTY
            };

            if src.0 != side.pieces.king.0 {
                pseudo_legals = pseudo_legals & allowed_non_king_moves
            }

            match pins[src.0.trailing_zeros() as usize] {
                Some(allowed) => pseudo_legals = pseudo_legals & allowed,
                None => ()
            }

            // Add back in en-passant.
            pseudo_legals = pseudo_legals | ep_mask;

            pseudo_legals.iter()
                .map(move |tgt| Move::Normal(src.into_coord(), tgt.into_coord()))
        });

    // Cloning the whole board probably not the most efficient.
    let occupancy_without_king = occupancy & (!side.pieces.king);

    // Remove king moves which would put the king in check.
    let moves_without_suicidal_king = mostly_legal_moves.filter(|m| {
        match m {
            Move::Normal(src, tgt) => {
                if *src == BitCoord(side.pieces.king.0).into_coord() {
                    !square_under_attack(occupancy_without_king, &other_side.pieces, *tgt, colour, mbb)
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
                        } else if state.en_passant.map(|ep| ep == tgt.into()).unwrap_or(false) {
                            // Remove en-passant if it would leave the king in check.
                            // Can't think of a better way to do this than just evaluating the new
                            // board for checks.
                            let mut new_pieces = side.pieces.clone();
                            let mut new_other_pieces = other_side.pieces.clone();
                            let taken_coord = match colour {
                                Colour::White => Into::<BitCoord>::into(tgt) >> 8,
                                Colour::Black => Into::<BitCoord>::into(tgt) << 8,
                            };

                            new_pieces.put_piece(Piece::Pawn, tgt.into());
                            new_pieces.clear_square(src.into());
                            new_other_pieces.clear_square(taken_coord);
                            let new_occupancy = new_pieces.all() | new_other_pieces.all();
                            if !square_under_attack(new_occupancy, &new_other_pieces, BitCoord(new_pieces.king.0).into_coord(), colour, mbb) {
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
    let home_rank_bb = match colour {
        Colour::White => BitBoard(0x00_00_00_00_00_00_00_FF),
        Colour::Black => BitBoard(0xFF_00_00_00_00_00_00_00),
    };

    let home_rank = match colour {
        Colour::White => 0x00,
        Colour::Black => 0x07,
    };

    if side.can_castle_queenside {
        if !(
            is_in_check ||
            occupancy & 0x70_70_70_70_70_70_70_70u64 & home_rank_bb != BitBoard::EMPTY ||
            square_under_attack(occupancy, &other_side.pieces, 0x20 | home_rank, colour, mbb) ||
            square_under_attack(occupancy, &other_side.pieces, 0x30 | home_rank, colour, mbb)
        ) {
            moves.push(Move::LongCastle);
        }
    }

    if side.can_castle_kingside {
        if !(
            is_in_check ||
            occupancy & 0x06_06_06_06_06_06_06_06u64 & home_rank_bb != BitBoard::EMPTY ||
            square_under_attack(occupancy, &other_side.pieces, 0x50 | home_rank, colour, mbb) ||
            square_under_attack(occupancy, &other_side.pieces, 0x60 | home_rank, colour, mbb)
        ) {
            moves.push(Move::Castle);
        }
    }

    moves
}

#[derive(Debug)]
enum Attack {
    Check(BitBoard),
    Pin(BitCoord, BitBoard),
}

pub fn square_under_attack(occupancy: BitBoard, other_pieces: &Pieces, coord: Coordinate, colour: Colour, mbb: &MagicBitBoards) -> bool {
    attacks_on_square(occupancy, other_pieces, coord.into(), colour, mbb)
        .iter()
        .any(|atk| match atk {
            Attack::Check(_) => true,
            _ => false,
        })
}

fn attacks_on_square(occupancy: BitBoard, other_pieces: &Pieces, coord: BitCoord, colour: Colour, mbb: &MagicBitBoards) -> Vec<Attack> {
    let mut attacks: Vec<Attack> = Vec::with_capacity(8);
    let other_occupancy = other_pieces.all();

    // Straight line pieces.
    [1, -1, 8, -8].iter().for_each(|dir| {
        let mut blocks = BitBoard::EMPTY;
        let mut pin: Option<BitCoord> = None;
        for c in Line::new(coord, *dir) {
            blocks = blocks | c;
            if (other_pieces.rooks | other_pieces.queens) & c != BitBoard::EMPTY {
                // Attack is real.
                match pin {
                    None => attacks.push(Attack::Check(blocks)),
                    Some(p) => attacks.push(Attack::Pin(p, blocks)),
                }
                break;
            } else if other_occupancy & c != BitBoard::EMPTY {
                // Blocked by opposing piece.
                break;
            } else if occupancy & c != BitBoard::EMPTY {
                // Blocked by friendly piece.
                if pin.is_some() {
                    // Two blocking pieces, no attack.
                    break;
                } else {
                    pin = Some(c);
                }
            }
        }
    });

    // Diagonal line pieces.
    [7, -7, 9, -9].iter().for_each(|dir| {
        let mut blocks = BitBoard::EMPTY;
        let mut pin: Option<BitCoord> = None;
        for c in Line::new(coord, *dir) {
            blocks = blocks | c;
            if (other_pieces.bishops | other_pieces.queens) & c != BitBoard::EMPTY {
                // Attack is real.
                match pin {
                    None => attacks.push(Attack::Check(blocks)),
                    Some(p) => attacks.push(Attack::Pin(p, blocks)),
                }
                break;
            } else if other_occupancy & c != BitBoard::EMPTY {
                // Blocked by opposing piece.
                break;
            } else if occupancy & c != BitBoard::EMPTY {
                // Blocked by friendly piece.
                if pin.is_some() {
                    // Two blocking pieces, no attack.
                    break;
                } else {
                    pin = Some(c);
                }
            }
        }
    });

    // Knights
    let knight_atks = mbb.knight(coord) & other_pieces.knights;
    if knight_atks != BitBoard::EMPTY {
        attacks.push(Attack::Check(knight_atks));
    }

    // King
    let king_atks = mbb.king(coord) & other_pieces.king;
    if king_atks != BitBoard::EMPTY {
        attacks.push(Attack::Check(king_atks));
    }

    // Pawns
    let pawn_atks = pawn_attacks(coord, colour) & other_pieces.pawns;
    if pawn_atks != BitBoard::EMPTY {
        attacks.push(Attack::Check(pawn_atks));
    }

    attacks
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
    let atks = pawn_attacks(coord, colour);
    let tgts = other_occupancy | en_passant.unwrap_or(BitCoord(0));
    let moves = match colour {
        Colour::White => {
            let home_row = BitBoard(0x00_00_00_00_00_00_FF_00);
            let mut moves = BitBoard::EMPTY;
            moves = moves | (coord << 8);
            if (home_row & coord != BitBoard::EMPTY) && (moves & occupancy == BitBoard::EMPTY) {
                moves = moves | (coord << 16);
            }
            moves & (!occupancy)
        },
        Colour::Black => {
            let home_row = BitBoard(0x00_FF_00_00_00_00_00_00);
            let mut moves = BitBoard::EMPTY;
            moves = moves | (coord >> 8);
            if (home_row & coord != BitBoard::EMPTY) && (moves & occupancy == BitBoard::EMPTY) {
                moves = moves | (coord >> 16);
            }

            moves & (!occupancy)
        },
    };

    moves | (atks & tgts)
}

fn pawn_attacks(coord: BitCoord, colour: Colour) -> BitBoard {
    match colour {
        Colour::White => {
            let mut atks = BitBoard::EMPTY;
            let file = (63 - coord.0.trailing_zeros()) % 8;
            if file != 0 {
                atks = atks | (coord << 9);
            }
            if file != 7 {
                atks = atks | (coord << 7);
            }

            atks
        },
        Colour::Black => {
            let mut atks = BitBoard::EMPTY;
            let file = (63 - coord.0.trailing_zeros()) % 8;
            if file != 0 {
                atks = atks | (coord >>  7);
            }
            if file != 7 {
                atks = atks | (coord >> 9);
            }

            atks
        },
    }
}

