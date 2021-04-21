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

    macro_rules! test_movement {
        [ $name:ident: Given $( a $colour:ident $piece:ident on $coord:expr ),+, the piece on $src:expr, can move to $( $tgt:expr ),+ ] => {
            #[test]
            fn $name() {
                let mut board = empty_board();
                $(
                    board[$coord.into_coord() as usize] = Square::Occupied(Colour::$colour, Piece::$piece);
                )+
                let mut expected: Vec<Coordinate> = Vec::new();
                $(
                    expected.push($tgt.into_coord());
                )+
                let moves = piece_movement(&board, $src.into_coord());
                let moves_set: HashSet<_> = moves.iter().collect();
                let expected_set: HashSet<_> = expected.iter().collect();
                assert_eq!(moves_set, expected_set);
            }
        };
        [ $name:ident: Given $( a $colour:ident $piece:ident on $coord:expr ),+, the piece on $src:expr, cannot move ] => {
            #[test]
            fn $name() {
                let mut board = empty_board();
                $(
                    board[$coord.into_coord() as usize] = Square::Occupied(Colour::$colour, Piece::$piece);
                )+
                let moves = piece_movement(&board, $src.into_coord());
                assert_eq!(moves, Vec::new());
            }
        };
    }

    test_movement![ white_pawn_moves_up:
        Given a White Pawn on "d4",
        the piece on "d4",
        can move to "d5"
    ];

    test_movement![ black_pawn_moves_down:
        Given a Black Pawn on "d4",
        the piece on "d4",
        can move to "d3"
    ];

    test_movement![ pawn_gets_blocked:
        Given a White Pawn on "d4", a Black Pawn on "d5",
        the piece on "d4",
        cannot move
    ];

    test_movement![ pawn_can_capture_opposite_colour:
        Given a White Pawn on "d4", a Black Knight on "e5",
        the piece on "d4",
        can move to "d5", "e5"
    ];

    test_movement![ pawn_cannot_capture_same_colour:
        Given a White Pawn on "d4", a White Knight on "e5",
        the piece on "d4",
        can move to "d5"
    ];

    test_movement![ black_pawn_can_capture_downwards:
        Given a Black Pawn on "d4", a White Knight on "c3",
        the piece on "d4",
        can move to "c3", "d3"
    ];

    test_movement![ king_unobstructed:
        Given a White King on "d4",
        the piece on "d4",
        can move to "e3", "e4", "e5", "d3", "d5", "c3", "c4", "c5"
    ];

    test_movement![ king_in_corner:
        Given a White King on "h8",
        the piece on "h8",
        can move to "h7", "g7", "g8"
    ];

    test_movement![ knight_unobstructed:
        Given a Black Knight on "d4",
        the piece on "d4",
        can move to "b5", "c6", "e6", "f5", "f3", "e2", "c2", "b3"
    ];

    test_movement![ knight_in_corner:
        Given a White Knight on "a1",
        the piece on "a1",
        can move to "b3", "c2"
    ];

    test_movement![ knight_blocked:
        Given a White Knight on "a1", a White Queen on "c2",
        the piece on "a1",
        can move to "b3"
    ];

    test_movement![ knight_capture:
        Given a Black Knight on "a1", a White Queen on "c2",
        the piece on "a1",
        can move to "b3", "c2"
    ];

    test_movement![ rook_unobstructed:
        Given a White Rook on "d4",
        the piece on "d4",
        can move to "a4", "b4", "c4", "e4", "f4", "g4", "h4", "d1", "d2", "d3", "d5", "d6", "d7", "d8"
    ];
}
