
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GameState {
    pub board: Board,
    pub active_colour: Colour,
    pub white: SideState,
    pub black: SideState,
    pub en_passant: Option<Coordinate>,
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

    pub fn clear_square(&mut self, coord: BitCoord) {
        self.king = self.king & (!coord);
        self.queens = self.queens & (!coord);
        self.rooks = self.rooks & (!coord);
        self.bishops = self.bishops & (!coord);
        self.knights = self.knights & (!coord);
        self.pawns = self.pawns & (!coord);
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
        self.king = self.king.move_if_present(from, to);
        self.queens = self.queens.move_if_present(from, to);
        self.rooks = self.rooks.move_if_present(from, to);
        self.bishops = self.bishops.move_if_present(from, to);
        self.knights = self.knights.move_if_present(from, to);
        self.pawns = self.pawns.move_if_present(from, to);
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

impl <T : Into<u32>> std::ops::Shr<T> for BitBoard {
    type Output = BitBoard;

    fn shr(self, rhs: T) -> Self::Output {
        BitBoard(self.0 >> rhs.into())
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
}

// Upper nibble = File
// Lower nibble = Rank
// i.e. b6 => (1, 5) => 0x15
pub type Coordinate = u8;

pub trait IntoCoord {
    fn into_coord(self) -> Coordinate;
}

// u64 with exactly 1 bit filled, representing a square.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BitCoord(pub u64);

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

impl <T : Into<u32>> std::ops::Shl<T> for BitCoord {
    type Output = BitCoord;

    fn shl(self, rhs: T) -> Self::Output {
        BitCoord(self.0 << rhs.into())
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
        BitCoord(1 << (rank * 8 + file))
    }
}

impl From<u64> for BitCoord {
    fn from(x: u64) -> BitCoord {
        BitCoord(x)
    }
}

impl From<Coordinate> for BitCoord {
    fn from(coord: Coordinate) -> BitCoord {
        (coord >> 4, coord & 0xF).into()
    }
}

impl IntoCoord for (u8, u8) {
    fn into_coord(self) -> Coordinate {
        let (file, rank) = self;
        ((file & 0xF) << 4) + (rank & 0xF)
    }
}

impl IntoCoord for BitCoord {
    fn into_coord(self) -> Coordinate {
        let zeros = self.0.trailing_zeros();
        let file = (zeros % 8) as u8;
        let rank = (zeros / 8) as u8;
        ((file & 0xF) << 4) + (rank & 0xF)
    }
}

impl <'a> IntoCoord for &'a str {
    fn into_coord(self) -> Coordinate {
        let mut chars = self.chars();
        let file = match chars.next().and_then(|c| c.to_digit(18)) {
            None => panic!("Malformed coordinate string: {}", self),
            Some(d) => d - 10,
        };
        let rank = match chars.next().and_then(|c| c.to_digit(10)) {
            None => panic!("Malformed coordinate string: {}", self),
            Some(d) => d - 1,
        };
        (file as u8, rank as u8).into_coord()
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum Move {
    Normal(Coordinate, Coordinate),
    Promotion(Coordinate, Coordinate, Piece),
    Castle,
    LongCastle,
}

#[cfg(test)]
mod test {
    use crate::types::*;

    #[test]
    fn str_to_coord() {
        assert_eq!("a1".into_coord(), 0x00);
        assert_eq!("d4".into_coord(), 0x33);
        assert_eq!("b7".into_coord(), 0x16);
        assert_eq!("h8".into_coord(), 0x77);
    }

    #[test]
    fn test_put_piece() {
        let mut pieces = Pieces::empty();
        pieces.put_piece(Piece::Knight, BitCoord(27));
        assert_eq!(pieces.get_piece(BitCoord(27)), Some(Piece::Knight));
    }
}
