use crate::board::{coord, directions, file, rank};
use crate::magic::MagicBitBoards;
use crate::moves::square_under_attack;
use crate::types::{BitCoord, Colour, Coordinate, GameState, IntoCoord, Move, Piece, Square};

impl GameState {
    pub fn make_move(&mut self, mv: Move) {
        self.fifty_move_clock += 1;

        match mv {
            Move::Normal(src, tgt) => {
                self.move_piece(src, tgt);
            },
            Move::Promotion(src, tgt, pc) => {
                self.move_piece(src, tgt);
                self.board[tgt as usize] = Square::Occupied(self.active_colour, pc);

                let active_side = match self.active_colour {
                    Colour::White => &mut self.white,
                    Colour::Black => &mut self.black,
                };
                active_side.pieces.remove_piece(pc, tgt.into());
                active_side.pieces.put_piece(pc, tgt.into());
            },
            Move::Castle => {
                match self.active_colour {
                    Colour::White => {
                        self.move_piece(0x40, 0x60);
                        self.move_piece(0x70, 0x50);
                    },
                    Colour::Black => {
                        self.move_piece(0x47, 0x67);
                        self.move_piece(0x77, 0x57);
                    },
                }
            },
            Move::LongCastle => {
                match self.active_colour {
                    Colour::White => {
                        self.move_piece(0x40, 0x20);
                        self.move_piece(0x00, 0x30);
                    },
                    Colour::Black => {
                        self.move_piece(0x47, 0x27);
                        self.move_piece(0x07, 0x37);
                    },
                }
            },
        }

        self.active_colour = match self.active_colour {
            Colour::White => Colour::Black,
            Colour::Black => Colour::White,
        }
    }

    pub fn is_in_check(&self, mbb: &MagicBitBoards) -> bool {
        let occupancy = self.white.pieces.all() | self.black.pieces.all();
        let (side, other_side) = match self.active_colour {
            Colour::White => (&self.white, &self.black),
            Colour::Black => (&self.black, &self.white),
        };
        let king = BitCoord::from(Into::<u64>::into(side.pieces.king)).into_coord();

        square_under_attack(occupancy, &other_side.pieces, king, self.active_colour, mbb)
    }

    fn move_piece(&mut self, src: Coordinate, tgt: Coordinate) {
        let colour = self.active_colour;
        let piece = match self.board[src as usize] {
            Square::Occupied(_, p) => p,
            Square::Empty => panic!("No piece on source square!"),
        };

        let (active_side, other_side) = match colour {
            Colour::White => (&mut self.white, &mut self.black),
            Colour::Black => (&mut self.black, &mut self.white),
        };

        let queenside_rook: Coordinate = match colour {
            Colour::White => 0x00,
            Colour::Black => 0x07,
        };

        let kingside_rook: Coordinate = match colour {
            Colour::White => 0x70,
            Colour::Black => 0x77,
        };

        let other_queenside_rook: Coordinate = match colour {
            Colour::White => 0x07,
            Colour::Black => 0x00,
        };

        let other_kingside_rook: Coordinate = match colour {
            Colour::White => 0x77,
            Colour::Black => 0x70,
        };

        let initial_king: Coordinate = match colour {
            Colour::White => 0x40,
            Colour::Black => 0x47,
        };

        let is_capture = self.board[tgt as usize] != Square::Empty;

        // Update board representations.
        self.board[tgt as usize] = self.board[src as usize];
        self.board[src as usize] = Square::Empty;

        active_side.pieces.move_piece(src.into(), tgt.into());
        other_side.pieces.clear_square(tgt.into());

        let tgt_bc: BitCoord = tgt.into();
        if piece == Piece::Pawn && self.en_passant.map(|ep| ep == tgt_bc).unwrap_or(false) {
            let taken_coord = tgt.wrapping_add(if colour == Colour::White { directions::DOWN } else { directions::UP });
            self.board[taken_coord as usize] = Square::Empty;
            other_side.pieces.clear_square(taken_coord.into());
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
        let src_rank = rank(src);
        let tgt_rank = rank(tgt);
        if piece == Piece::Pawn && (tgt_rank == src_rank + 2 || src_rank == tgt_rank + 2) {
            self.en_passant = Some((file(src), (src_rank + tgt_rank) / 2).into());
        } else {
            self.en_passant = None;
        }

        // Adjust clocks.
        if piece == Piece::Pawn || is_capture {
            self.fifty_move_clock = 0;
        }
    }
}
