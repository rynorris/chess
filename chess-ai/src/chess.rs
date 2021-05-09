use chess_lib::magic::MagicBitBoards;
use chess_lib::moves::legal_moves;
use chess_lib::types as chess;

use crate::minimax;
use crate::montecarlo;

pub struct Chess<'a> {
    pub state: chess::GameState,
    pub mbb: &'a MagicBitBoards,
}

impl <'a> Chess<'a> {
    pub fn new(state: chess::GameState, mbb: &'a MagicBitBoards) -> Chess<'a> {
        Chess{
            state,
            mbb,
        }
    }
}

impl <'a> Clone for Chess<'a> {
    fn clone(&self) -> Chess<'a> {
        Chess::new(self.state.clone(), &self.mbb)
    }
}

impl <'a> minimax::Game for Chess<'a> {
    type Move = chess::Move;

    fn make_move(&mut self, mv: Self::Move) {
        self.state.make_move(mv);
    }

    fn legal_moves(&self) -> Vec<Self::Move> {
        legal_moves(&self.state, &self.mbb)
    }

    fn zobrist_hash(&self) -> chess::ZobristHash {
        self.state.zh
    }
}

impl <'a> montecarlo::Game for Chess<'a> {
    type Move = chess::Move;

    fn make_move(&mut self, mv: Self::Move) {
        self.state.make_move(mv);
    }

    fn legal_moves(&self) -> Vec<Self::Move> {
        legal_moves(&self.state, &self.mbb)
    }

    fn game_state(&self) -> montecarlo::GameState {
        let num_moves = self.legal_moves().len();
        if num_moves == 0 {
            if self.state.is_in_check(&self.mbb) {
                // Checkmate.
                montecarlo::GameState::Finished(montecarlo::GameResult::Loss)
            } else {
                // Stalemate.
                montecarlo::GameState::Finished(montecarlo::GameResult::Draw)
            }
        } else if self.state.fifty_move_clock >= 50 {
            // Fifty move rule.
            montecarlo::GameState::Finished(montecarlo::GameResult::Draw)
        } else {
            montecarlo::GameState::Ongoing
        }
    }

    fn active_player(&self) -> montecarlo::Player {
        match self.state.active_colour {
            chess::Colour::White => montecarlo::Player::One,
            chess::Colour::Black => montecarlo::Player::Two,
        }
    }
}

