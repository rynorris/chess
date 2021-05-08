
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GameState {
    pub active_colour: Colour,
    pub white: SideState,
    pub black: SideState,
    pub en_passant: Option<BitCoord>,
    pub fifty_move_clock: u8,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SideState {
    pub pieces: Pieces,
    pub can_castle_kingside: bool,
    pub can_castle_queenside: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pieces {
    pub king: BitBoard,
    pub queens: BitBoard,
    pub rooks: BitBoard,
    pub bishops: BitBoard,
    pub knights: BitBoard,
    pub pawns: BitBoard,
}

impl Pieces {
    pub fn empty() -> Pieces {
        Pieces {
            king: BitBoard(0),
            queens: BitBoard(0),
            rooks: BitBoard(0),
            bishops: BitBoard(0),
            knights: BitBoard(0),
            pawns: BitBoard(0),
        }
    }

    pub fn all(&self) -> BitBoard {
        self.king | self.queens | self.rooks | self.bishops | self.knights | self.pawns
    }

    pub fn put_piece(&mut self, piece: Piece, coord: BitCoord) {
        let bb: &mut BitBoard = self.piece_bb(piece);
        *bb = *bb | coord;
    }

    pub fn remove_piece(&mut self, piece: Piece, coord: BitCoord) {
        let bb: &mut BitBoard = self.piece_bb(piece);
        *bb = *bb & (!coord);
    }

    pub fn clear_square(&mut self, coord: BitCoord) {
        let mask = !coord;
        self.king = self.king & mask;
        self.queens = self.queens & mask;
        self.rooks = self.rooks & mask;
        self.bishops = self.bishops & mask;
        self.knights = self.knights & mask;
        self.pawns = self.pawns & mask;
    }

    pub fn get_piece(&self, coord: BitCoord) -> Option<Piece> {
        if self.king & coord != BitBoard::EMPTY {
            Some(Piece::King)
        } else if self.queens & coord != BitBoard::EMPTY {
            Some(Piece::Queen)
        } else if self.rooks & coord != BitBoard::EMPTY {
            Some(Piece::Rook)
        } else if self.bishops & coord != BitBoard::EMPTY {
            Some(Piece::Bishop)
        } else if self.knights & coord != BitBoard::EMPTY {
            Some(Piece::Knight)
        } else if self.pawns & coord != BitBoard::EMPTY {
            Some(Piece::Pawn)
        } else {
            None
        }
    }

    pub fn move_piece(&mut self, from: BitCoord, to: BitCoord) {
        let pc = self.get_piece(from).unwrap();
        self.remove_piece(pc, from);
        self.put_piece(pc, to);
    }

    fn piece_bb(&mut self, piece: Piece) -> &mut BitBoard {
        match piece {
            Piece::King => &mut self.king,
            Piece::Queen => &mut self.queens,
            Piece::Rook => &mut self.rooks,
            Piece::Bishop => &mut self.bishops,
            Piece::Knight => &mut self.knights,
            Piece::Pawn => &mut self.pawns,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Colour {
    White,
    Black,
}

impl Colour {
    pub fn other(c: Colour) -> Colour {
        match c {
            Colour::White => Colour::Black,
            Colour::Black => Colour::White,
        }
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum Piece {
    King,
    Queen,
    Rook,
    Knight,
    Pawn,
    Bishop,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Square {
    Empty,
    Occupied(Colour, Piece),
}

// Note: because of how our coordinates work, we need 120 slots here.
//       (max coord is 0x77 = 119)
// Only 64 will ever be filled.
pub type Board = [Square; 120];

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct BitBoard(pub u64);

impl <T : Into<u64>> std::ops::BitAnd<T> for BitBoard {
    type Output = BitBoard;

    fn bitand(self, rhs: T) -> Self::Output {
        BitBoard(self.0 & rhs.into())
    }
}

impl <T : Into<u64>> std::ops::BitOr<T> for BitBoard {
    type Output = BitBoard;

    fn bitor(self, rhs: T) -> Self::Output {
        BitBoard(self.0 | rhs.into())
    }
}

impl std::ops::Not for BitBoard {
    type Output = BitBoard;

    fn not(self) -> Self::Output {
        BitBoard(!self.0)
    }
}

impl std::ops::Shr<u32> for BitBoard {
    type Output = BitBoard;

    fn shr(self, rhs: u32) -> Self::Output {
        let Self(lhs) = self;
        Self(lhs >> rhs)
    }
}

impl Into<u64> for BitBoard {
    fn into(self) -> u64 {
        self.0
    }
}

impl std::fmt::Display for BitBoard {
    fn fmt(&self, w: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        for c in 0..64 {
            if c % 8 == 0 {
                write!(w, "\n")?;
            }

            if self.0 & (1 << (63 - c)) == 0 {
                write!(w, ".")?;
            } else {
                write!(w, "o")?;
            }
        }
        Ok(())
    }
}

impl BitBoard {
    pub const EMPTY: BitBoard = BitBoard(0);

    pub fn move_if_present(self, from: BitCoord, to: BitCoord) -> BitBoard {
        if self & from != BitBoard(0) {
            self & (!from) | to
        } else {
            self
        }
    }

    pub fn iter(self) -> BitBoardIter {
        BitBoardIter{bb: self, c: BitCoord(1)}
    }
}

pub struct BitBoardIter {
    bb: BitBoard,
    c: BitCoord,
}

impl Iterator for BitBoardIter {
    type Item = BitCoord;

    fn next(&mut self) -> Option<BitCoord> {
        if self.bb == BitBoard::EMPTY {
            None
        } else {
            let zeros = self.bb.0.trailing_zeros();
            self.c = self.c << zeros;
            self.bb = self.bb >> zeros;

            let ret = Some(self.c);
            self.c = self.c << 1u32;
            self.bb = self.bb >> 1u32;

            ret
        }
    }
}

// u64 with exactly 1 bit filled, representing a square.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct BitCoord(pub u64);

impl BitCoord {
    pub fn rank(self) -> u32 {
        self.0.trailing_zeros() / 8
    }

    pub fn file(self) -> u32 {
        (63 - self.0.trailing_zeros()) % 8
    }
}

impl <T : Into<u64>> std::ops::BitAnd<T> for BitCoord {
    type Output = BitCoord;

    fn bitand(self, rhs: T) -> Self::Output {
        BitCoord(self.0 & rhs.into())
    }
}

impl <T : Into<u64>> std::ops::BitOr<T> for BitCoord {
    type Output = BitCoord;

    fn bitor(self, rhs: T) -> Self::Output {
        BitCoord(self.0 | rhs.into())
    }
}

impl std::ops::Not for BitCoord {
    type Output = BitCoord;

    fn not(self) -> Self::Output {
        BitCoord(!self.0)
    }
}

impl std::ops::Shl<u32> for BitCoord {
    type Output = BitCoord;

    fn shl(self, rhs: u32) -> Self::Output {
        BitCoord(self.0 << rhs)
    }
}

impl std::ops::Shr<u32> for BitCoord {
    type Output = BitCoord;

    fn shr(self, rhs: u32) -> Self::Output {
        BitCoord(self.0 >> rhs)
    }
}

impl Into<u64> for BitCoord {
    fn into(self) -> u64 {
        self.0
    }
}

impl From<(u8, u8)> for BitCoord {
    fn from(file_and_rank: (u8, u8)) -> BitCoord {
        let (file, rank) = file_and_rank;
        BitCoord(1 << (rank * 8 + 7 - file))
    }
}

impl From<(u32, u32)> for BitCoord {
    fn from(file_and_rank: (u32, u32)) -> BitCoord {
        let (file, rank) = file_and_rank;
        BitCoord(1 << (rank * 8 + 7 - file))
    }
}

impl From<u64> for BitCoord {
    fn from(x: u64) -> BitCoord {
        BitCoord(x)
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum Move {
    Normal(BitCoord, BitCoord),
    Promotion(BitCoord, BitCoord, Piece),
    Castle,
    LongCastle,
}

#[cfg(test)]
mod test {
    use crate::types::*;

    #[test]
    fn test_put_piece() {
        let mut pieces = Pieces::empty();
        pieces.put_piece(Piece::Knight, BitCoord(27));
        assert_eq!(pieces.get_piece(BitCoord(27)), Some(Piece::Knight));
    }
}
