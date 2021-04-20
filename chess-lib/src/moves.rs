use crate::types::{Board, Colour, Coordinate, Piece, Square};

type Direction = u8;
type Line = (Direction, fn (c: Coordinate) -> u8);

const LEFT: Line = (255, file);
const RIGHT: Line = (1, file);
const UP: Line = (8, rank);
const DOWN: Line = (248, rank);
const UP_LEFT: Line = (7, down_diagonal);
const UP_RIGHT: Line = (9, up_diagonal);
const DOWN_LEFT: Line = (247, up_diagonal);
const DOWN_RIGHT: Line = (249, down_diagonal);

const STRAIGHTS: [Line; 4] = [LEFT, RIGHT, UP, DOWN];
const DIAGONALS: [Line; 4] = [UP_LEFT, UP_RIGHT, DOWN_LEFT, DOWN_RIGHT];

pub fn legal_moves(board: &Board, coord: Coordinate) -> Vec<Coordinate> {
    let mut moves: Vec<Coordinate> = Vec::with_capacity(32);

    let (piece, colour) = match board[coord as usize] {
        Square::Occupied(p, c) => (p, c),
        Square::Empty => panic!("No piece on square {}", coord),
    };

    match piece {
        Piece::King => {
            STRAIGHTS.as_ref().into_iter().for_each(|line| {
                moves.append(&mut moves_in_line(&board, coord, colour, *line, 1));
            });
            DIAGONALS.as_ref().into_iter().for_each(|line| {
                moves.append(&mut moves_in_line(&board, coord, colour, *line, 1));
            });
        },
        Piece::Queen => {
            STRAIGHTS.as_ref().into_iter().for_each(|line| {
                moves.append(&mut moves_in_line(&board, coord, colour, *line, 8));
            });
            DIAGONALS.as_ref().into_iter().for_each(|line| {
                moves.append(&mut moves_in_line(&board, coord, colour, *line, 8));
            });
        },
        Piece::Rook => {
            STRAIGHTS.as_ref().into_iter().for_each(|line| {
                moves.append(&mut moves_in_line(&board, coord, colour, *line, 8));
            });
        },
        Piece::Bishop => {
            DIAGONALS.as_ref().into_iter().for_each(|line| {
                moves.append(&mut moves_in_line(&board, coord, colour, *line, 8));
            });
        },
        Piece::Knight => {
        },
        Piece::Pawn => {
            moves.append(&mut pawn_moves(&board, coord, colour));
        },
    };

    moves
}

fn moves_in_line(board: &Board, coord: Coordinate, colour: Colour, line: Line, limit: usize) -> Vec<Coordinate> {
    let line_moves = generate_line_with_limit(coord, line, limit);
    truncate_blocked(board, colour, line_moves)
}

fn pawn_moves(board: &Board, coord: Coordinate, colour: Colour) -> Vec<Coordinate> {
    let (fwd, d1, d2) = match colour {
        Colour::White => (UP, UP_LEFT, UP_RIGHT),
        Colour::Black => (DOWN, DOWN_LEFT, DOWN_RIGHT),
    };

    let moves: Vec<Coordinate> = Vec::with_capacity(3);

    moves.extend(generate_line_with_limit(coord, fwd, 1).into_iter().filter(|m| can_move_to(board, colour, *m)));
    moves.extend(generate_line_with_limit(coord, d1, 1).into_iter().filter(|m| can_capture(board, colour, *m)));
    moves.extend(generate_line_with_limit(coord, d2, 1).into_iter().filter(|m| can_capture(board, colour, *m)));

    moves
}

fn knight_moves(board: &Board, coord: Coordinate, colour: Colour) -> Vec<Coordinate> {
}

fn can_move_to(board: &Board, colour: Colour, coord: Coordinate) -> bool {
        match board[coord as usize] {
            Square::Empty => true,
            Square::Occupied(_, _) => false,
        }
}

fn can_capture(board: &Board, colour: Colour, coord: Coordinate) -> bool {
        match board[coord as usize] {
            Square::Empty => false,
            Square::Occupied(_, c) => c != colour,
        }
}

fn truncate_blocked(board: &Board, colour: Colour, mut moves: Vec<Coordinate>) -> Vec<Coordinate> {
    let mut ix: usize = 0;
    while ix < moves.len() {
        match board[moves[ix] as usize] {
            Square::Empty => ix += 1,
            Square::Occupied(_, c) => {
                if *c != colour {
                    ix += 1;
                }
                break;
            },
        }
    }
    moves.truncate(ix);
    moves
}

fn generate_line(coord: Coordinate, line: Line) -> Vec<Coordinate> {
    generate_line_with_limit(coord, line, 8)
}

fn generate_line_with_limit(mut coord: Coordinate, line: Line, limit: usize) -> Vec<Coordinate> {
    let mut coords: Vec<Coordinate> = Vec::with_capacity(7);
    let (step, invariant) = line;
    let invariant_val = invariant(coord);

    while coords.len() < limit {
        coord = coord.wrapping_add(step);
        if coord / 64 != 0 || invariant(coord) != invariant_val {
            break
        }
        coords.push(coord);
    }

    coords
}

fn rank(coord: Coordinate) -> u8 {
    coord % 8
}

fn file(coord: Coordinate) -> u8 {
    coord / 8
}

fn up_diagonal(coord: Coordinate) -> u8 {
    rank(coord).wrapping_sub(file(coord))
}

fn down_diagonal(coord: Coordinate) -> u8 {
    rank(coord).wrapping_add(file(coord))
}

#[cfg(test)]
mod tests {
    use crate::moves::*;

    #[test]
    fn line_right() {
        assert_eq!(generate_line(0, RIGHT), vec![1, 2, 3, 4, 5, 6, 7]);
        assert_eq!(generate_line(5, RIGHT), vec![6, 7]);
        assert_eq!(generate_line(7, RIGHT), vec![]);
        assert_eq!(generate_line(58, RIGHT), vec![59, 60, 61, 62, 63]);
    }

    #[test]
    fn line_left() {
        assert_eq!(generate_line(0, LEFT), vec![]);
        assert_eq!(generate_line(5, LEFT), vec![4, 3, 2, 1, 0]);
        assert_eq!(generate_line(7, LEFT), vec![6, 5, 4, 3, 2, 1, 0]);
        assert_eq!(generate_line(58, LEFT), vec![57, 56]);
    }

    #[test]
    fn line_up() {
        assert_eq!(generate_line(0, UP), vec![8, 16, 24, 32, 40, 48, 56]);
        assert_eq!(generate_line(7, UP), vec![15, 23, 31, 39, 47, 55, 63]);
        assert_eq!(generate_line(37, UP), vec![45, 53, 61]);
        assert_eq!(generate_line(62, UP), vec![]);
    }

    #[test]
    fn line_down() {
        assert_eq!(generate_line(0, DOWN), vec![]);
        assert_eq!(generate_line(7, DOWN), vec![]);
        assert_eq!(generate_line(37, DOWN), vec![29, 21, 13, 5]);
        assert_eq!(generate_line(62, DOWN), vec![54, 46, 38, 30, 22, 14, 6]);
    }

    #[test]
    fn line_up_right() {
        assert_eq!(generate_line(0, UP_RIGHT), vec![9, 18, 27, 36, 45, 54, 63]);
        assert_eq!(generate_line(7, UP_RIGHT), vec![]);
        assert_eq!(generate_line(12, UP_RIGHT), vec![21, 30, 39]);
        assert_eq!(generate_line(40, UP_RIGHT), vec![49, 58]);
    }

    #[test]
    fn line_up_left() {
        assert_eq!(generate_line(0, UP_LEFT), vec![]);
        assert_eq!(generate_line(7, UP_LEFT), vec![14, 21, 28, 35, 42, 49, 56]);
        assert_eq!(generate_line(12, UP_LEFT), vec![19, 26, 33, 40]);
        assert_eq!(generate_line(54, UP_LEFT), vec![61]);
    }

    #[test]
    fn line_down_right() {
        assert_eq!(generate_line(0, DOWN_RIGHT), vec![]);
        assert_eq!(generate_line(17, DOWN_RIGHT), vec![10, 3]);
        assert_eq!(generate_line(56, DOWN_RIGHT), vec![49, 42, 35, 28, 21, 14, 7]);
        assert_eq!(generate_line(61, DOWN_RIGHT), vec![54, 47]);
    }

    #[test]
    fn line_down_left() {
        assert_eq!(generate_line(0, DOWN_LEFT), vec![]);
        assert_eq!(generate_line(17, DOWN_LEFT), vec![8]);
        assert_eq!(generate_line(63, DOWN_LEFT), vec![54, 45, 36, 27, 18, 9, 0]);
        assert_eq!(generate_line(23, DOWN_LEFT), vec![14, 5]);
    }
}
