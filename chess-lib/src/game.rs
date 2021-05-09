use crate::magic::MagicBitBoards;
use crate::moves::square_under_attack;
use crate::types::{BitBoard, BitCoord, Colour, GameState, Move, Piece, SideState, ZobristHash};
use crate::zobrist::ZobristHasher;

impl GameState {
    pub fn new(
        active_colour: Colour, 
        white: SideState,
        black: SideState,
        en_passant: Option<BitCoord>,
        fifty_move_clock: u8,
    ) -> GameState {
        let mut state = GameState{
            active_colour,
            white,
            black,
            en_passant,
            fifty_move_clock,
            zh: ZobristHash(0),
        };

        state.zh = ZobristHasher::default().hash(&state);

        state
    }

    pub fn make_move(&mut self, mv: Move) {
        let hasher = ZobristHasher::default();

        self.fifty_move_clock += 1;

        match mv {
            Move::Normal(piece, src, tgt) => {
                self.move_piece(piece, src, tgt, hasher);
            },
            Move::Promotion(src, tgt, pc) => {
                self.move_piece(Piece::Pawn, src, tgt, hasher);
                self.remove_active_piece(Piece::Pawn, tgt, hasher);
                self.put_active_piece(pc, tgt, hasher);
            },
            Move::Castle => {
                match self.active_colour {
                    Colour::White => {
                        self.remove_active_piece(Piece::King, BitCoord(0x00_00_00_00_00_00_00_08), hasher);
                        self.remove_active_piece(Piece::Rook, BitCoord(0x00_00_00_00_00_00_00_01), hasher);
                        self.put_active_piece(Piece::King, BitCoord(0x00_00_00_00_00_00_00_02), hasher);
                        self.put_active_piece(Piece::Rook, BitCoord(0x00_00_00_00_00_00_00_04), hasher);
                        self.disable_active_kingside_castle(hasher);
                        self.disable_active_queenside_castle(hasher);
                        self.clear_en_passant(hasher);
                    },
                    Colour::Black => {
                        self.remove_active_piece(Piece::King, BitCoord(0x08_00_00_00_00_00_00_00), hasher);
                        self.remove_active_piece(Piece::Rook, BitCoord(0x01_00_00_00_00_00_00_00), hasher);
                        self.put_active_piece(Piece::King, BitCoord(0x02_00_00_00_00_00_00_00), hasher);
                        self.put_active_piece(Piece::Rook, BitCoord(0x04_00_00_00_00_00_00_00), hasher);
                        self.disable_active_kingside_castle(hasher);
                        self.disable_active_queenside_castle(hasher);
                        self.clear_en_passant(hasher);
                    },
                }
            },
            Move::LongCastle => {
                match self.active_colour {
                    Colour::White => {
                        self.remove_active_piece(Piece::King, BitCoord(0x00_00_00_00_00_00_00_08), hasher);
                        self.remove_active_piece(Piece::Rook, BitCoord(0x00_00_00_00_00_00_00_80), hasher);
                        self.put_active_piece(Piece::King, BitCoord(0x00_00_00_00_00_00_00_20), hasher);
                        self.put_active_piece(Piece::Rook, BitCoord(0x00_00_00_00_00_00_00_10), hasher);
                        self.disable_active_kingside_castle(hasher);
                        self.disable_active_queenside_castle(hasher);
                        self.clear_en_passant(hasher);
                    },
                    Colour::Black => {
                        self.remove_active_piece(Piece::King, BitCoord(0x08_00_00_00_00_00_00_00), hasher);
                        self.remove_active_piece(Piece::Rook, BitCoord(0x80_00_00_00_00_00_00_00), hasher);
                        self.put_active_piece(Piece::King, BitCoord(0x20_00_00_00_00_00_00_00), hasher);
                        self.put_active_piece(Piece::Rook, BitCoord(0x10_00_00_00_00_00_00_00), hasher);
                        self.disable_active_kingside_castle(hasher);
                        self.disable_active_queenside_castle(hasher);
                        self.clear_en_passant(hasher);
                    },
                }
            },
        }

        self.active_colour = Colour::other(self.active_colour);
        self.zh = hasher.toggle_active_colour(self.zh);
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

    fn move_piece(&mut self, piece: Piece, src: BitCoord, tgt: BitCoord, hasher: &ZobristHasher) {
        let colour = self.active_colour;

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

        let is_pawn = piece == Piece::Pawn;

        self.remove_active_piece(piece, src, hasher);
        self.put_active_piece(piece, tgt, hasher);
        let mut is_capture = match self.other_side_mut().pieces.get_piece(tgt) {
            Some(pc) => {
                self.remove_other_piece(pc, tgt, hasher);
                true
            },
            None => false,
        };

        if is_pawn && self.en_passant.map(|ep| ep == tgt).unwrap_or(false) {
            let taken_coord = match colour {
                Colour::White => tgt >> 8,
                Colour::Black => tgt << 8,
            };
            self.remove_other_piece(Piece::Pawn, taken_coord, hasher);
            is_capture = true;
        }

        // King moves.
        if piece == Piece::King {
            self.disable_active_kingside_castle(hasher);
            self.disable_active_queenside_castle(hasher);
        }

        if src == queenside_rook {
            self.disable_active_queenside_castle(hasher);
        } else if src == kingside_rook {
            self.disable_active_kingside_castle(hasher);
        }

        if tgt == other_queenside_rook {
            self.disable_other_queenside_castle(hasher);
        } else if tgt == other_kingside_rook {
            self.disable_other_kingside_castle(hasher);
        }

        // Check for En Passant.
        if is_pawn && (src == tgt << 16 || src == tgt >> 16) {
            let ep = match colour {
                Colour::White => tgt >> 8,
                Colour::Black => tgt << 8,
            };
            self.set_en_passant(ep, hasher);
        } else {
            self.clear_en_passant(hasher);
        }

        // Adjust clocks.
        if is_pawn || is_capture {
            self.fifty_move_clock = 0;
        }
    }

    fn put_active_piece(&mut self, piece: Piece, coord: BitCoord, hasher: &ZobristHasher) {
        self.active_side_mut().pieces.put_piece(piece, coord);
        self.zh = hasher.toggle_piece(self.zh, self.active_colour, piece, coord);
    }

    fn remove_active_piece(&mut self, piece: Piece, coord: BitCoord, hasher: &ZobristHasher) {
        self.active_side_mut().pieces.remove_piece(piece, coord);
        self.zh = hasher.toggle_piece(self.zh, self.active_colour, piece, coord);
    }

    fn remove_other_piece(&mut self, piece: Piece, coord: BitCoord, hasher: &ZobristHasher) {
        self.other_side_mut().pieces.remove_piece(piece, coord);
        self.zh = hasher.toggle_piece(self.zh, Colour::other(self.active_colour), piece, coord);
    }

    fn set_en_passant(&mut self, coord: BitCoord, hasher: &ZobristHasher) {
        match self.en_passant.replace(coord) {
            Some(prev) => self.zh = hasher.toggle_en_passant(self.zh, prev),
            None => (),
        };
        self.zh = hasher.toggle_en_passant(self.zh, coord);
    }

    fn clear_en_passant(&mut self, hasher: &ZobristHasher) {
        match self.en_passant.take() {
            Some(coord) => self.zh = hasher.toggle_en_passant(self.zh, coord),
            None => (),
        };
    }

    fn disable_active_queenside_castle(&mut self, hasher: &ZobristHasher) {
        self.active_side_mut().can_castle_queenside = false;
        self.zh = match self.active_colour {
            Colour::White => hasher.toggle_white_queenside(self.zh),
            Colour::Black => hasher.toggle_black_queenside(self.zh),
        };
    }

    fn disable_active_kingside_castle(&mut self, hasher: &ZobristHasher) {
        self.active_side_mut().can_castle_kingside = false;
        self.zh = match self.active_colour {
            Colour::White => hasher.toggle_white_kingside(self.zh),
            Colour::Black => hasher.toggle_black_kingside(self.zh),
        };
    }

    fn disable_other_queenside_castle(&mut self, hasher: &ZobristHasher) {
        self.other_side_mut().can_castle_queenside = false;
        self.zh = match self.active_colour {
            Colour::White => hasher.toggle_black_queenside(self.zh),
            Colour::Black => hasher.toggle_white_queenside(self.zh),
        };
    }

    fn disable_other_kingside_castle(&mut self, hasher: &ZobristHasher) {
        self.other_side_mut().can_castle_kingside = false;
        self.zh = match self.active_colour {
            Colour::White => hasher.toggle_black_kingside(self.zh),
            Colour::Black => hasher.toggle_white_kingside(self.zh),
        };
    }

    fn active_side_mut(&mut self) -> &mut SideState {
        match self.active_colour {
            Colour::White => &mut self.white,
            Colour::Black => &mut self.black,
        }
    }

    fn other_side_mut(&mut self) -> &mut SideState {
        match self.active_colour {
            Colour::White => &mut self.black,
            Colour::Black => &mut self.white,
        }
    }
}
