use crate::types::{Board, Colour, Coordinate, Piece, Square};


pub fn empty_board() -> Board {
    [Square::Empty; 120]
}

#[macro_export]
macro_rules! board {
    [ $( $coord:expr => $colour:ident $piece:ident ),* ] => {
        {
            let mut b = empty_board();
            $(
                b[$coord.into_coord() as usize] = Square::Occupied(crate::types::Colour::$colour, crate::types::Piece::$piece);
            )*
            b
        }
     };
}

pub fn print_board(board: &Board) {
    for minus_rank in 0..=7 {
        let rank = 7 - minus_rank;
        for file in 0..=7 {
            let square_char: char = match board[coord(file, rank) as usize] {
                Square::Empty => '.',
                Square::Occupied(Colour::White, Piece::King) => 'K',
                Square::Occupied(Colour::White, Piece::Queen) => 'Q',
                Square::Occupied(Colour::White, Piece::Rook) => 'R',
                Square::Occupied(Colour::White, Piece::Bishop) => 'B',
                Square::Occupied(Colour::White, Piece::Knight) => 'N',
                Square::Occupied(Colour::White, Piece::Pawn) => 'P',
                Square::Occupied(Colour::Black, Piece::King) => 'k',
                Square::Occupied(Colour::Black, Piece::Queen) => 'q',
                Square::Occupied(Colour::Black, Piece::Rook) => 'r',
                Square::Occupied(Colour::Black, Piece::Bishop) => 'b',
                Square::Occupied(Colour::Black, Piece::Knight) => 'n',
                Square::Occupied(Colour::Black, Piece::Pawn) => 'p',
            };
            print!("{}", square_char);
        }
        print!("\n");
    }
}

pub type Direction = u8;

pub mod directions {
    use crate::board::Direction;

    pub const RIGHT: Direction = 0x10;
    pub const LEFT: Direction = 0u8.wrapping_sub(RIGHT);
    pub const UP: Direction = 0x01;
    pub const DOWN: Direction = 0u8.wrapping_sub(UP);
    pub const UP_RIGHT: Direction = UP.wrapping_add(RIGHT);
    pub const UP_LEFT: Direction = UP.wrapping_add(LEFT);
    pub const DOWN_RIGHT: Direction = DOWN.wrapping_add(RIGHT);
    pub const DOWN_LEFT: Direction = DOWN.wrapping_add(LEFT);

    pub const STRAIGHTS: [Direction; 4] = [LEFT, RIGHT, UP, DOWN];
    pub const DIAGONALS: [Direction; 4] = [UP_LEFT, UP_RIGHT, DOWN_LEFT, DOWN_RIGHT];
    pub const ALL: [Direction; 8] = [LEFT, RIGHT, UP, DOWN, UP_LEFT, UP_RIGHT, DOWN_LEFT, DOWN_RIGHT];

    pub const KNIGHT: [Direction; 8] = [
        RIGHT.wrapping_add(UP.wrapping_mul(2)),
        RIGHT.wrapping_add(DOWN.wrapping_mul(2)),
        LEFT.wrapping_add(UP.wrapping_mul(2)),
        LEFT.wrapping_add(DOWN.wrapping_mul(2)),
        UP.wrapping_add(LEFT.wrapping_mul(2)),
        UP.wrapping_add(RIGHT.wrapping_mul(2)),
        DOWN.wrapping_add(LEFT.wrapping_mul(2)),
        DOWN.wrapping_add(RIGHT.wrapping_mul(2)),
    ];

    pub fn is_straight(dir: Direction) -> bool {
        dir == RIGHT || dir == LEFT || dir == UP || dir == DOWN
    }

    pub fn is_diagonal(dir: Direction) -> bool {
        dir == UP_RIGHT || dir == UP_LEFT || dir == DOWN_RIGHT || dir == DOWN_LEFT
    }

    pub fn reverse(dir: Direction) -> Direction {
        0u8.wrapping_sub(dir)
    }
}

pub struct Line {
    cur: Coordinate,
    dir: Direction,
}

impl Line {
    pub fn new(cur: Coordinate, dir: Direction) -> Line {
        Line { cur, dir }
    }
}

impl Iterator for Line {
    type Item = Coordinate;

    fn next(&mut self) -> Option<Coordinate> {
        let next = self.cur.wrapping_add(self.dir);

        if !is_in_bounds(next) {
            None
        } else {
            self.cur = next;
            Some(next)
        }
    }
}

#[inline]
pub const fn is_in_bounds(coord: Coordinate) -> bool {
    coord & 0x88 == 0
}

#[inline]
pub const fn rank(coord: Coordinate) -> u8 {
    coord & 0xF
}

#[inline]
pub const fn file(coord: Coordinate) -> u8 {
    (coord >> 4) & 0xF
}

#[inline]
pub const fn coord(file: u8, rank: u8) -> Coordinate {
    ((file & 0xF) << 4) | (rank & 0xF)
}

#[cfg(test)]
mod tests {
    use crate::board::*;
    use crate::board::directions::*;

    fn generate_line(orig: Coordinate, dir: Direction) -> Vec<Coordinate> {
        let line = Line::new(orig, dir);
        let mut coords: Vec<Coordinate> = Vec::with_capacity(8);
        coords.extend(line);
        coords
    }

    #[test]
    fn coord_from_rank_and_file() {
        assert_eq!(coord(0, 0), 0x00);
        assert_eq!(coord(3, 5), 0x35);
        assert_eq!(coord(6, 7), 0x67);
    }

    #[test]
    fn line_right() {
        assert_eq!(generate_line(0x00, RIGHT), vec![0x10, 0x20, 0x30, 0x40, 0x50, 0x60, 0x70]);
        assert_eq!(generate_line(0x50, RIGHT), vec![0x60, 0x70]);
        assert_eq!(generate_line(0x70, RIGHT), vec![]);
        assert_eq!(generate_line(0x27, RIGHT), vec![0x37, 0x47, 0x57, 0x67, 0x77]);
    }

    #[test]
    fn line_left() {
        assert_eq!(generate_line(0x00, LEFT), vec![]);
        assert_eq!(generate_line(0x50, LEFT), vec![0x40, 0x30, 0x20, 0x10, 0x00]);
        assert_eq!(generate_line(0x70, LEFT), vec![0x60, 0x50, 0x40, 0x30, 0x20, 0x10, 0x00]);
        assert_eq!(generate_line(0x27, LEFT), vec![0x17, 0x07]);
    }

    #[test]
    fn line_up() {
        assert_eq!(generate_line(0x00, UP), vec![0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07]);
        assert_eq!(generate_line(0x70, UP), vec![0x71, 0x72, 0x73, 0x74, 0x75, 0x76, 0x77]);
        assert_eq!(generate_line(0x45, UP), vec![0x46, 0x47]);
        assert_eq!(generate_line(0x67, UP), vec![]);
    }

    #[test]
    fn line_down() {
        assert_eq!(generate_line(0x00, DOWN), vec![]);
        assert_eq!(generate_line(0x70, DOWN), vec![]);
        assert_eq!(generate_line(0x45, DOWN), vec![0x44, 0x43, 0x42, 0x41, 0x40]);
        assert_eq!(generate_line(0x67, DOWN), vec![0x66, 0x65, 0x64, 0x63, 0x62, 0x61, 0x60]);
    }

    #[test]
    fn line_up_right() {
        assert_eq!(generate_line(0x00, UP_RIGHT), vec![0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77]);
        assert_eq!(generate_line(0x70, UP_RIGHT), vec![]);
        assert_eq!(generate_line(0x41, UP_RIGHT), vec![0x52, 0x63, 0x74]);
        assert_eq!(generate_line(0x05, UP_RIGHT), vec![0x16, 0x27]);
    }

    #[test]
    fn line_up_left() {
        assert_eq!(generate_line(0x00, UP_LEFT), vec![]);
        assert_eq!(generate_line(0x70, UP_LEFT), vec![0x61, 0x52, 0x43, 0x34, 0x25, 0x16, 0x07]);
        assert_eq!(generate_line(0x41, UP_LEFT), vec![0x32, 0x23, 0x14, 0x05]);
        assert_eq!(generate_line(0x66, UP_LEFT), vec![0x57]);
    }

    #[test]
    fn line_down_right() {
        assert_eq!(generate_line(0x00, DOWN_RIGHT), vec![]);
        assert_eq!(generate_line(0x12, DOWN_RIGHT), vec![0x21, 0x30]);
        assert_eq!(generate_line(0x07, DOWN_RIGHT), vec![0x16, 0x25, 0x34, 0x43, 0x52, 0x61, 0x70]);
        assert_eq!(generate_line(0x57, DOWN_RIGHT), vec![0x66, 0x75]);
    }

    #[test]
    fn line_down_left() {
        assert_eq!(generate_line(0x00, DOWN_LEFT), vec![]);
        assert_eq!(generate_line(0x12, DOWN_LEFT), vec![0x01]);
        assert_eq!(generate_line(0x77, DOWN_LEFT), vec![0x66, 0x55, 0x44, 0x33, 0x22, 0x11, 0x00]);
        assert_eq!(generate_line(0x72, DOWN_LEFT), vec![0x61, 0x50]);
    }
}
