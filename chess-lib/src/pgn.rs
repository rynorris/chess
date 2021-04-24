use std::fmt;
use std::fmt::Display;
use crate::board::{file, rank};
use crate::fmt::{format_file, format_rank, format_piece};
use crate::types::{Coordinate, GameState, Move, Piece, Square};

pub enum PGNMove {
    Normal(PGNMoveData),
    Castle(bool, bool),
    LongCastle(bool, bool),
}

pub struct PGNMoveData {
    pub piece: Piece,
    pub disambiguate_file: Option<u8>,
    pub disambiguate_rank: Option<u8>,
    pub to_square: Coordinate,
    pub is_capture: bool,
    pub is_check: bool,
    pub is_checkmate: bool,
    pub promote_to: Option<Piece>,
}

impl PGNMove {
    pub fn from_internal(state: &GameState, mv: Move) -> PGNMove {
        let mut new_state = state.clone();
        new_state.make_move(mv);

        let is_check = new_state.is_in_check();

        match mv {
            Move::Normal(src, tgt) => {
                let piece = match state.board[src as usize] {
                    Square::Occupied(_, p) => p,
                    _ => panic!("Source square is empty"),
                };

                let is_capture = match state.board[tgt as usize] {
                    Square::Occupied(_, _) => true,
                    _ => false,
                };

                PGNMove::Normal(PGNMoveData{
                    piece,
                    to_square: tgt,
                    is_capture,
                    is_check,
                    is_checkmate: false,
                    disambiguate_file: None,
                    disambiguate_rank: None,
                    promote_to: None,
                })
            },
            Move::Promotion(src, tgt, promote_to) => {
                let piece = match state.board[src as usize] {
                    Square::Occupied(_, p) => p,
                    _ => panic!("Source square is empty"),
                };

                let is_capture = match state.board[tgt as usize] {
                    Square::Occupied(_, _) => true,
                    _ => false,
                };

                PGNMove::Normal(PGNMoveData{
                    piece,
                    to_square: tgt,
                    is_capture,
                    is_check,
                    is_checkmate: false,
                    disambiguate_file: None,
                    disambiguate_rank: None,
                    promote_to: Some(promote_to),
                })
            },
            Move::Castle => PGNMove::Castle(is_check, false),
            Move::LongCastle => PGNMove::LongCastle(is_check, false),
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            PGNMove::Normal(data) => {
                let mut s: String = String::new();
                if data.piece != Piece::Pawn {
                    s.push(format_piece(data.piece));
                }

                if data.is_capture {
                    s.push('x');
                }

                s.push(format_file(file(data.to_square)));
                s.push(format_rank(rank(data.to_square)));

                match data.promote_to {
                    Some(p) => {
                        s.push('=');
                        s.push(format_piece(p));
                    },
                    None => (),
                }

                if data.is_check {
                    s.push('+');
                }

                if data.is_checkmate {
                    s.push('#');
                }

                s
            },
            PGNMove::Castle(check, checkmate) => {
                let mut s = "O-O".to_owned();

                if *check {
                    s.push('+');
                }

                if *checkmate {
                    s.push('#');
                }

                s
            },
            PGNMove::LongCastle(check, checkmate) => {
                let mut s = "O-O-O".to_owned();

                if *check {
                    s.push('+');
                }

                if *checkmate {
                    s.push('#');
                }

                s
            },
        }
    }
}

impl Display for PGNMove {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}
