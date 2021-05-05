use chess_lib::moves::legal_moves;
use chess_lib::types as chess;

use crate::montecarlo::{Game, GameResult, GameState, Player};


impl Game for chess::GameState {
    type Move = chess::Move;

    fn make_move(&mut self, mv: Self::Move) {
        self.make_move(mv);
    }

    fn legal_moves(&self) -> Vec<Self::Move> {
        legal_moves(self)
    }

    fn game_state(&self) -> GameState {
        let num_moves = self.legal_moves().len();
        if num_moves == 0 {
            if self.is_in_check() {
                // Checkmate.
                GameState::Finished(GameResult::Loss)
            } else {
                // Stalemate.
                GameState::Finished(GameResult::Draw)
            }
        } else if self.fifty_move_clock >= 50 {
            // Fifty move rule.
            GameState::Finished(GameResult::Draw)
        } else {
            GameState::Ongoing
        }
    }

    fn active_player(&self) -> Player {
        match self.active_colour {
            chess::Colour::White => Player::One,
            chess::Colour::Black => Player::Two,
        }
    }
}

