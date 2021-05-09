use rand::prelude::*;
use rand_chacha::ChaCha8Rng;

use crate::types::{BitBoard, BitCoord, Colour, GameState, Piece, ZobristHash};

static mut DEFAULT_HASHER: Option<ZobristHasher> = None;


#[derive(Clone, Copy, Debug)]
pub struct ZobristHasher {
    numbers: [u64; 781],
}

impl ZobristHasher {
    const DEFAULT_SEED: u64 = 26355;
    const BLACK_TO_MOVE: usize = 12 * 64;
    const WHITE_QUEENSIDE: usize = 12 * 64 + 1;
    const WHITE_KINGSIDE: usize = 12 * 64 + 1;
    const BLACK_QUEENSIDE: usize = 12 * 64 + 2;
    const BLACK_KINGSIDE: usize = 12 * 64 + 3;
    const EN_PASSANT: usize = 12 * 64 + 4;

    pub fn default() -> &'static ZobristHasher {
        unsafe {
            DEFAULT_HASHER.get_or_insert_with(|| Self::from_seed(Self::DEFAULT_SEED))
        }
    }

    pub fn from_seed(seed: u64) -> ZobristHasher {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let mut numbers = [0u64; 781];
        for ix in 0..781 {
            numbers[ix] = rng.gen();
        }

        ZobristHasher{numbers}
    }

    pub fn identify_diff(&self, lhs: ZobristHash, rhs: ZobristHash) -> Option<usize> {
        let diff = lhs.0 ^ rhs.0;
        for ix in 0..781 {
            if diff == self.numbers[ix] {
                return Some(ix);
            }
        }
        None
    }

    pub fn hash(&self, state: &GameState) -> ZobristHash {
        let mut zh = ZobristHash(0);

        zh = self.toggle_pieces(zh, Colour::White, Piece::King, state.white.pieces.king);
        zh = self.toggle_pieces(zh, Colour::White, Piece::Queen, state.white.pieces.queens);
        zh = self.toggle_pieces(zh, Colour::White, Piece::Rook, state.white.pieces.rooks);
        zh = self.toggle_pieces(zh, Colour::White, Piece::Bishop, state.white.pieces.bishops);
        zh = self.toggle_pieces(zh, Colour::White, Piece::Knight, state.white.pieces.knights);
        zh = self.toggle_pieces(zh, Colour::White, Piece::Pawn, state.white.pieces.pawns);

        zh = self.toggle_pieces(zh, Colour::Black, Piece::King, state.black.pieces.king);
        zh = self.toggle_pieces(zh, Colour::Black, Piece::Queen, state.black.pieces.queens);
        zh = self.toggle_pieces(zh, Colour::Black, Piece::Rook, state.black.pieces.rooks);
        zh = self.toggle_pieces(zh, Colour::Black, Piece::Bishop, state.black.pieces.bishops);
        zh = self.toggle_pieces(zh, Colour::Black, Piece::Knight, state.black.pieces.knights);
        zh = self.toggle_pieces(zh, Colour::Black, Piece::Pawn, state.black.pieces.pawns);

        if state.active_colour == Colour::Black {
            zh = self.toggle_active_colour(zh);
        }

        if state.white.can_castle_queenside {
            zh = self.toggle_white_queenside(zh);
        }

        if state.white.can_castle_kingside {
            zh = self.toggle_white_kingside(zh);
        }

        if state.black.can_castle_queenside {
            zh = self.toggle_black_queenside(zh);
        }

        if state.black.can_castle_kingside {
            zh = self.toggle_black_kingside(zh);
        }

        match state.en_passant {
            Some(ep) => zh = self.toggle_en_passant(zh, ep),
            None => (),
        }

        zh
    }

    pub fn toggle_active_colour(&self, zh: ZobristHash) -> ZobristHash {
        zh ^ self.numbers[Self::BLACK_TO_MOVE]
    }

    pub fn toggle_piece(&self, zh: ZobristHash, colour: Colour, piece: Piece, coord: BitCoord) -> ZobristHash {
       zh ^ self.numbers[Self::piece_index(colour, piece, coord)]
    }

    pub fn toggle_en_passant(&self, zh: ZobristHash, en_passant: BitCoord) -> ZobristHash {
        let ep_file = en_passant.file() as usize;
        let ep_file_ix = Self::EN_PASSANT + ep_file;
        zh ^ self.numbers[ep_file_ix]
    }

    pub fn toggle_white_queenside(&self, zh: ZobristHash) -> ZobristHash {
        zh ^ self.numbers[Self::WHITE_QUEENSIDE]
    }

    pub fn toggle_white_kingside(&self, zh: ZobristHash) -> ZobristHash {
        zh ^ self.numbers[Self::WHITE_KINGSIDE]
    }

    pub fn toggle_black_queenside(&self, zh: ZobristHash) -> ZobristHash {
        zh ^ self.numbers[Self::BLACK_QUEENSIDE]
    }

    pub fn toggle_black_kingside(&self, zh: ZobristHash) -> ZobristHash {
        zh ^ self.numbers[Self::BLACK_KINGSIDE]
    }

    fn toggle_pieces(&self, mut zh: ZobristHash, colour: Colour, piece: Piece, bb: BitBoard) -> ZobristHash {
        for coord in bb.iter() {
           zh = self.toggle_piece(zh, colour, piece, coord);
        }
        zh
    }

    fn piece_index(colour: Colour, piece: Piece, coord: BitCoord) -> usize {
        let coord_ix: usize = coord.0.trailing_zeros() as usize;

        let piece_ix: usize = match piece {
            Piece::King => 0 * 64,
            Piece::Queen => 1 * 64,
            Piece::Rook => 2 * 64,
            Piece::Bishop => 3 * 64,
            Piece::Knight => 4 * 64,
            Piece::Pawn => 5 * 64,
        };

        let colour_ix: usize = match colour {
            Colour::White => 0 * 6 * 64,
            Colour::Black => 1 * 6 * 64,
        };

        colour_ix + piece_ix + coord_ix
    }
}

#[cfg(test)]
mod tests {
    use crate::fen::*;
    use crate::zobrist::*;

    #[test]
    fn from_scratch() {
        // Tests that the hasher does something, and remains deterministic.
        let hasher = ZobristHasher::default();
        let state = load_fen(STARTING_POSITION);
        let zh = hasher.hash(&state);
        assert_eq!(zh, ZobristHash(0x5aba4aaed7d93fd));
    }
}

