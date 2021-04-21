use crate::board::{directions, is_in_bounds, Line};
use crate::types::{Board, Colour, Coordinate, Piece, Square};

pub fn piece_movement(board: &Board, coord: Coordinate) -> Vec<Coordinate> {
    let mut moves: Vec<Coordinate> = Vec::with_capacity(32);

    let (colour, piece) = match board[coord as usize] {
        Square::Occupied(p, c) => (p, c),
        Square::Empty => panic!("No piece on square {}", coord),
    };

    match piece {
        Piece::King => {
            directions::ALL.as_ref().into_iter().for_each(|dir| {
                moves.append(&mut moves_in_line(&board, colour, Line::new(coord, *dir), 1));
            });
        },
        Piece::Queen => {
            directions::ALL.as_ref().into_iter().for_each(|dir| {
                moves.append(&mut moves_in_line(&board, colour, Line::new(coord, *dir), 8));
            });
        },
        Piece::Rook => {
            directions::STRAIGHTS.as_ref().into_iter().for_each(|dir| {
                moves.append(&mut moves_in_line(&board, colour, Line::new(coord, *dir), 8));
            });
        },
        Piece::Bishop => {
            directions::DIAGONALS.as_ref().into_iter().for_each(|dir| {
                moves.append(&mut moves_in_line(&board, colour, Line::new(coord, *dir), 8));
            });
        },
        Piece::Knight => {
            moves.append(&mut knight_moves(&board, coord, colour));
        },
        Piece::Pawn => {
            moves.append(&mut pawn_moves(&board, coord, colour));
        },
    };

    moves
}

fn moves_in_line(board: &Board, colour: Colour, line: Line, limit: usize) -> Vec<Coordinate> {
    until_blocked(board, colour, &mut line.take(limit))
}

fn pawn_moves(board: &Board, coord: Coordinate, colour: Colour) -> Vec<Coordinate> {
    let (fwd, d1, d2) = match colour {
        Colour::White => (directions::UP, directions::UP_LEFT, directions::UP_RIGHT),
        Colour::Black => (directions::DOWN, directions::DOWN_LEFT, directions::DOWN_RIGHT),
    };

    let mut moves: Vec<Coordinate> = Vec::with_capacity(3);

    moves.extend(Line::new(coord, fwd).take(1).filter(|m| can_move_to(board, *m)));
    moves.extend(Line::new(coord, d1).take(1).filter(|m| can_capture(board, colour, *m)));
    moves.extend(Line::new(coord, d2).take(1).filter(|m| can_capture(board, colour, *m)));

    moves
}

fn knight_moves(board: &Board, coord: Coordinate, colour: Colour) -> Vec<Coordinate> {
    let mut moves: Vec<Coordinate> = Vec::with_capacity(8);

    moves.extend(
        directions::KNIGHT
            .as_ref()
            .into_iter()
            .map(|d| coord.wrapping_add(*d))
            .filter(|m| is_in_bounds(*m))
            .filter(|m| can_move_to(board, *m) || can_capture(board, colour, *m))
    );

    moves
}

fn can_move_to(board: &Board, coord: Coordinate) -> bool {
        match board[coord as usize] {
            Square::Empty => true,
            Square::Occupied(_, _) => false,
        }
}

fn can_capture(board: &Board, colour: Colour, coord: Coordinate) -> bool {
        match board[coord as usize] {
            Square::Empty => false,
            Square::Occupied(c, _) => c != colour,
        }
}

fn until_blocked(board: &Board, colour: Colour, moves_iter: &mut dyn Iterator<Item=Coordinate>) -> Vec<Coordinate> {
    let mut moves: Vec<Coordinate> = Vec::with_capacity(7);

    loop {
        match moves_iter.next() {
            Some(m) => {
                match board[m as usize] {
                    Square::Empty => moves.push(m),
                    Square::Occupied(c, _) => {
                        if c != colour {
                            moves.push(m);
                        }
                        break;
                    },
                };
            },
            None => break,
        }
    }

    moves
}

#[cfg(test)]
mod test {
    use std::collections::HashSet;
    use crate::board::{empty_board};
    use crate::moves::*;
    use crate::types::IntoCoord;

    macro_rules! coords {
        [ $( $coord:expr ),* ] => {
            vec![
                $(
                    $coord.into_coord(),
                )*
            ]
        };
    }

    macro_rules! test_movement {
        [ $name:ident: $board:expr, $coord: expr => $expected: expr ] => {
            #[test]
            fn $name() {
                let board = $board;
                let expected: Vec<Coordinate> = $expected;
                let moves = piece_movement(&board, $coord.into_coord());
                let moves_set: HashSet<_> = moves.iter().collect();
                let expected_set: HashSet<_> = expected.iter().collect();
                assert_eq!(moves_set, expected_set);
            }
        };
    }

    test_movement![ white_pawn_moves_up: board!["d4" => White Pawn], "d4" => coords!["d5"] ];
    test_movement![ black_pawn_moves_down: board!["d4" => Black Pawn], "d4" => coords!["d3"] ];

    test_movement![ pawn_gets_blocked:
        board!["d4" => White Pawn, "d5" => White Knight], "d4"
        => coords![]
    ];

    test_movement![ pawn_can_capture_opposite_colour:
        board!["d4" => White Pawn, "e5" => Black Knight], "d4"
        => coords!["d5", "e5"]
    ];

    test_movement![ pawn_cannot_capture_same_colour:
        board!["d4" => White Pawn, "e5" => White Knight], "d4"
        => coords!["d5"]
    ];

    test_movement![ black_pawn_can_capture_downwards:
        board!["d4" => Black Pawn, "c3" => White Knight], "d4"
        => coords!["c3", "d3"]
    ];
}
