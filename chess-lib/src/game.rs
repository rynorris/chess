use crate::magic::MagicBitBoards;
use crate::moves::square_under_attack;
use crate::types::{BitBoard, BitCoord, Colour, GameState, Move, Piece};

impl GameState {
    pub fn make_move(&mut self, mv: Move) {
        self.fifty_move_clock += 1;

        match mv {
            Move::Normal(src, tgt) => {
                self.move_piece(src, tgt);
            },
            Move::Promotion(src, tgt, pc) => {
                self.move_piece(src, tgt);

                let active_side = match self.active_colour {
                    Colour::White => &mut self.white,
                    Colour::Black => &mut self.black,
                };
                active_side.pieces.remove_piece(Piece::Pawn, tgt.into());
                active_side.pieces.put_piece(pc, tgt.into());
            },
            Move::Castle => {
                match self.active_colour {
                    Colour::White => {
                        self.white.pieces.remove_piece(Piece::King, BitCoord(0x00_00_00_00_00_00_00_08));
                        self.white.pieces.remove_piece(Piece::Rook, BitCoord(0x00_00_00_00_00_00_00_01));
                        self.white.pieces.put_piece(Piece::King, BitCoord(0x00_00_00_00_00_00_00_02));
                        self.white.pieces.put_piece(Piece::Rook, BitCoord(0x00_00_00_00_00_00_00_04));
                        self.white.can_castle_kingside = false;
                        self.white.can_castle_queenside = false;
                        self.en_passant = None;
                    },
                    Colour::Black => {
                        self.black.pieces.remove_piece(Piece::King, BitCoord(0x08_00_00_00_00_00_00_00));
                        self.black.pieces.remove_piece(Piece::Rook, BitCoord(0x01_00_00_00_00_00_00_00));
                        self.black.pieces.put_piece(Piece::King, BitCoord(0x02_00_00_00_00_00_00_00));
                        self.black.pieces.put_piece(Piece::Rook, BitCoord(0x04_00_00_00_00_00_00_00));
                        self.black.can_castle_kingside = false;
                        self.black.can_castle_queenside = false;
                        self.en_passant = None;
                    },
                }
            },
            Move::LongCastle => {
                match self.active_colour {
                    Colour::White => {
                        self.white.pieces.remove_piece(Piece::King, BitCoord(0x00_00_00_00_00_00_00_08));
                        self.white.pieces.remove_piece(Piece::Rook, BitCoord(0x00_00_00_00_00_00_00_80));
                        self.white.pieces.put_piece(Piece::King, BitCoord(0x00_00_00_00_00_00_00_20));
                        self.white.pieces.put_piece(Piece::Rook, BitCoord(0x00_00_00_00_00_00_00_10));
                        self.white.can_castle_kingside = false;
                        self.white.can_castle_queenside = false;
                        self.en_passant = None;
                    },
                    Colour::Black => {
                        self.black.pieces.remove_piece(Piece::King, BitCoord(0x08_00_00_00_00_00_00_00));
                        self.black.pieces.remove_piece(Piece::Rook, BitCoord(0x80_00_00_00_00_00_00_00));
                        self.black.pieces.put_piece(Piece::King, BitCoord(0x20_00_00_00_00_00_00_00));
                        self.black.pieces.put_piece(Piece::Rook, BitCoord(0x10_00_00_00_00_00_00_00));
                        self.black.can_castle_kingside = false;
                        self.black.can_castle_queenside = false;
                        self.en_passant = None;
                    },
                }
            },
        }

        self.active_colour = Colour::other(self.active_colour);
    }

    pub fn is_in_check(&self, mbb: &MagicBitBoards) -> bool {
        let occupancy = self.white.pieces.all() | self.black.pieces.all();
        let (side, other_side) = match self.active_colour {
            Colour::White => (&self.white, &self.black),
            Colour::Black => (&self.black, &self.white),
        };
        let king = BitCoord(side.pieces.king.0);

        square_under_attack(occupancy, &other_side.pieces, king, self.active_colour, mbb)
    }

    pub fn find_piece(&self, coord: BitCoord) -> Option<(Colour, Piece)> {
        match self.white.pieces.get_piece(coord) {
            Some(pc) => return Some((Colour::White, pc)),
            None => (),
        };

        match self.black.pieces.get_piece(coord) {
            Some(pc) => return Some((Colour::Black, pc)),
            None => (),
        };

        None
    }

    fn move_piece(&mut self, src: BitCoord, tgt: BitCoord) {
        let colour = self.active_colour;

        let (active_side, other_side) = match colour {
            Colour::White => (&mut self.white, &mut self.black),
            Colour::Black => (&mut self.black, &mut self.white),
        };


        let home_rank = match colour {
            Colour::White => BitBoard(0x00_00_00_00_00_00_00_FF),
            Colour::Black => BitBoard(0xFF_00_00_00_00_00_00_00),
        };

        let other_home_rank = match colour {
            Colour::White => BitBoard(0xFF_00_00_00_00_00_00_00),
            Colour::Black => BitBoard(0x00_00_00_00_00_00_00_FF),
        };

        let queenside_rook: BitCoord = BitCoord(home_rank.0 & 0x80_00_00_00_00_00_00_80);
        let kingside_rook: BitCoord = BitCoord(home_rank.0 & 0x01_00_00_00_00_00_00_01);
        let other_queenside_rook: BitCoord = BitCoord(other_home_rank.0 & 0x80_00_00_00_00_00_00_80);
        let other_kingside_rook: BitCoord = BitCoord(other_home_rank.0 & 0x01_00_00_00_00_00_00_01);
        let initial_king: BitCoord = BitCoord(home_rank.0 & 0x08_00_00_00_00_00_00_08);

        let is_capture = other_side.pieces.all() & tgt != BitBoard::EMPTY;
        let is_pawn = active_side.pieces.pawns & src != BitBoard::EMPTY;

        active_side.pieces.move_piece(src.into(), tgt.into());
        other_side.pieces.clear_square(tgt.into());

        if is_pawn && self.en_passant.map(|ep| ep == tgt).unwrap_or(false) {
            let taken_coord = match colour {
                Colour::White => tgt >> 8,
                Colour::Black => tgt << 8,
            };
            other_side.pieces.remove_piece(Piece::Pawn, taken_coord);
        }

        if src == initial_king {
            active_side.can_castle_queenside = false;
            active_side.can_castle_kingside = false;
        }

        if src == queenside_rook {
            active_side.can_castle_queenside = false;
        } else if src == kingside_rook {
            active_side.can_castle_kingside = false;
        }

        if tgt == other_queenside_rook {
            other_side.can_castle_queenside = false;
        } else if tgt == other_kingside_rook {
            other_side.can_castle_kingside = false;
        }

        // Check for En Passant.
        if is_pawn && (src == tgt << 16 || src == tgt >> 16) {
            self.en_passant = match colour {
                Colour::White => Some(tgt >> 8),
                Colour::Black => Some(tgt << 8),
            };
        } else {
            self.en_passant = None;
        }

        // Adjust clocks.
        if is_pawn || is_capture {
            self.fifty_move_clock = 0;
        }
    }
}
