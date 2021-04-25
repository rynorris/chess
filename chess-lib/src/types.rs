
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GameState {
    pub board: Board,
    pub active_colour: Colour,
    pub white: SideState,
    pub black: SideState,
    pub en_passant: Option<Coordinate>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SideState {
    pub king_coord: Coordinate,
    pub can_castle_kingside: bool,
    pub can_castle_queenside: bool,
}

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
    Occupied(Colour, Piece),
}

// Note: because of how our coordinates work, we need 120 slots here.
//       (max coord is 0x77 = 119)
// Only 64 will ever be filled.
pub type Board = [Square; 120];

// Upper nibble = File
// Lower nibble = Rank
// i.e. b6 => (1, 5) => 0x15
pub type Coordinate = u8;

pub trait IntoCoord {
    fn into_coord(self) -> Coordinate;
}

impl IntoCoord for (u8, u8) {
    fn into_coord(self) -> Coordinate {
        let (file, rank) = self;
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
}
