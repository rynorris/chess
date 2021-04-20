
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Colour {
    White,
    Black,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
    Occupied(Piece, Colour),
}

pub type Board = [Square; 64];

// (Rank, File), 0-indexed.
// Coordinate = Rank * 8 + File
pub type Coordinate = u8;

pub enum Move {
    Normal(NormalMove),
    Castle,
    LongCastle,
}

pub struct NormalMove {
    pub origin: Coordinate,
    pub target: Coordinate,
    pub promote_to: Option<Piece>,
}
