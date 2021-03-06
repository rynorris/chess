use crate::magic::{Line, MagicBitBoards};
use crate::types::{BitBoard, BitCoord, Colour, GameState, Move, Piece, Pieces};

pub fn legal_moves(state: &GameState, mbb: &MagicBitBoards) -> Vec<Move> {
    let colour = state.active_colour;

    let (side, other_side) = match colour {
        Colour::White => (&state.white, &state.black),
        Colour::Black => (&state.black, &state.white),
    };

    // Calculate restrictions on allowed moves based on checks and pins.
    let active_occupancy = side.pieces.all();
    let other_occupancy = other_side.pieces.all();
    let occupancy = active_occupancy | other_occupancy;

    // This is a map from a pinned piece, to the coordinates it can move to without
    // breaking the pin.
    // i.e. if a piece is in this map, it can ONLY move to the listed squares.
    let mut pins: [Option<BitBoard>; 64] = [None; 64];

    let allowed_non_king_moves = attacks_on_square(
        &mut pins,
        occupancy,
        &other_side.pieces,
        BitCoord(side.pieces.king.0),
        colour,
        mbb,
    );

    let is_in_check = allowed_non_king_moves != BitBoard(0xFF_FF_FF_FF_FF_FF_FF_FF);

    // Now get all moves disregarding restrictions.
    let mostly_legal_moves = side.pieces.all()
        .iter()
        .flat_map(|src| {
            let piece = side.pieces.get_piece(src).expect("No piece on square");
            let mut pseudo_legals = magic_piece_movement(
                piece,
                active_occupancy,
                other_occupancy,
                occupancy,
                src,
                colour,
                state.en_passant,
            mbb);
            
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
                .map(move |tgt| Move::Normal(piece, src, tgt))
        });

    // Cloning the whole board probably not the most efficient.
    let occupancy_without_king = occupancy & (!side.pieces.king);

    // Remove king moves which would put the king in check.
    let moves_without_suicidal_king = mostly_legal_moves.filter(|m| {
        match m {
            Move::Normal(piece, _, tgt) => {
                if *piece == Piece::King {
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
            Move::Normal(piece, src, tgt) => {
                let is_pawn = piece == Piece::Pawn;
                if is_pawn {
                    // Expand pawn moves to last rank.
                    // Don't have to check colours since pawns can't move backwards.
                    if BitBoard(0xFF_00_00_00_00_00_00_FF) & tgt != BitBoard::EMPTY {
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
                        if !square_under_attack(new_occupancy, &new_other_pieces, BitCoord(new_pieces.king.0), colour, mbb) {
                            moves.push(m);
                        }
                    } else {
                        moves.push(m);
                    }
                } else {
                    moves.push(m);
                }
            },
            _ => panic!("Should only have normal moves at this stage"),
        }
    }


    // Add castling if legal.
    let home_rank = match colour {
        Colour::White => BitBoard(0x00_00_00_00_00_00_00_FF),
        Colour::Black => BitBoard(0xFF_00_00_00_00_00_00_00),
    };

    if side.can_castle_queenside {
        if !(
            is_in_check ||
            occupancy & 0x70_70_70_70_70_70_70_70u64 & home_rank != BitBoard::EMPTY ||
            square_under_attack(occupancy, &other_side.pieces, BitCoord(0x20_00_00_00_00_00_00_20) & home_rank, colour, mbb) ||
            square_under_attack(occupancy, &other_side.pieces, BitCoord(0x10_00_00_00_00_00_00_10) & home_rank, colour, mbb)
        ) {
            moves.push(Move::LongCastle);
        }
    }

    if side.can_castle_kingside {
        if !(
            is_in_check ||
            occupancy & 0x06_06_06_06_06_06_06_06u64 & home_rank != BitBoard::EMPTY ||
            square_under_attack(occupancy, &other_side.pieces, BitCoord(0x04_00_00_00_00_00_00_04) & home_rank, colour, mbb) ||
            square_under_attack(occupancy, &other_side.pieces, BitCoord(0x02_00_00_00_00_00_00_02) & home_rank, colour, mbb)
        ) {
            moves.push(Move::Castle);
        }
    }

    moves
}

#[derive(Debug)]
struct Pin(BitCoord, BitBoard);

pub fn square_under_attack(occupancy: BitBoard, other_pieces: &Pieces, coord: BitCoord, colour: Colour, mbb: &MagicBitBoards) -> bool {
    let straight_atks = mbb.rook(coord).lookup(occupancy) & (other_pieces.rooks | other_pieces.queens);
    if straight_atks != BitBoard::EMPTY {
        return true;
    }

    let diag_atks = mbb.bishop(coord).lookup(occupancy) & (other_pieces.bishops | other_pieces.queens);
    if diag_atks != BitBoard::EMPTY {
        return true;
    }

    // Knights
    let knight_atks = mbb.knight(coord) & other_pieces.knights;
    if knight_atks != BitBoard::EMPTY {
        return true;
    }

    // King
    let king_atks = mbb.king(coord) & other_pieces.king;
    if king_atks != BitBoard::EMPTY {
        return true;
    }

    // Pawns
    let pawn_atks = pawn_attacks(coord, colour) & other_pieces.pawns;
    if pawn_atks != BitBoard::EMPTY {
        return true;
    }

    false
}

fn attacks_on_square(
    pins: &mut [Option<BitBoard>; 64],
    occupancy: BitBoard,
    other_pieces: &Pieces,
    coord: BitCoord,
    colour: Colour,
    mbb: &MagicBitBoards,
) -> BitBoard {
    let mut allowed_moves = BitBoard(0xFF_FF_FF_FF_FF_FF_FF_FF);
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
                    None => allowed_moves = allowed_moves & blocks,
                    Some(p) => pins[p.0.trailing_zeros() as usize] = Some(blocks),
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
                    None => allowed_moves = allowed_moves & blocks,
                    Some(p) => pins[p.0.trailing_zeros() as usize] = Some(blocks),
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
        allowed_moves = allowed_moves & knight_atks;
    }

    // King
    let king_atks = mbb.king(coord) & other_pieces.king;
    if king_atks != BitBoard::EMPTY {
        allowed_moves = allowed_moves & king_atks;
    }

    // Pawns
    let pawn_atks = pawn_attacks(coord, colour) & other_pieces.pawns;
    if pawn_atks != BitBoard::EMPTY {
        allowed_moves = allowed_moves & pawn_atks;
    }

    allowed_moves
}

fn magic_piece_movement(
    piece: Piece,
    active_occupancy: BitBoard,
    other_occupancy: BitBoard,
    occupancy: BitBoard,
    coord: BitCoord,
    colour: Colour,
    en_passant: Option<BitCoord>,
    mbb: &MagicBitBoards,
) -> BitBoard {
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
    moves & (!active_occupancy)
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

