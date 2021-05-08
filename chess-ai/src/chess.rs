use chess_lib::magic::MagicBitBoards;
use chess_lib::moves::legal_moves;
use chess_lib::types as chess;

use crate::montecarlo::{Game, GameResult, GameState, Player};

pub struct Chess {
    pub state: chess::GameState,
    pub mbb: MagicBitBoards,
}

impl Chess {
    pub fn new(state: chess::GameState) -> Chess {
        Chess{
            state,
            mbb: MagicBitBoards::default(),
        }
    }
}

impl Clone for Chess {
    fn clone(&self) -> Chess {
        Chess::new(self.state.clone())
    }
}

impl Game for Chess {
    type Move = chess::Move;

    fn make_move(&mut self, mv: Self::Move) {
        self.state.make_move(mv);
    }

    fn legal_moves(&self) -> Vec<Self::Move> {
        legal_moves(&self.state, &self.mbb)
    }

    fn game_state(&self) -> GameState {
        let num_moves = self.legal_moves().len();
        if num_moves == 0 {
            if self.state.is_in_check(&self.mbb) {
                // Checkmate.
                GameState::Finished(GameResult::Loss)
            } else {
                // Stalemate.
                GameState::Finished(GameResult::Draw)
            }
        } else if self.state.fifty_move_clock >= 50 {
            // Fifty move rule.
            GameState::Finished(GameResult::Draw)
        } else {
            GameState::Ongoing
        }
    }

    fn active_player(&self) -> Player {
        match self.state.active_colour {
            chess::Colour::White => Player::One,
            chess::Colour::Black => Player::Two,
        }
    }
}

