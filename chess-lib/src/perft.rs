use crate::fen::{load_fen, STARTING_POSITION};
use crate::moves::legal_moves;
use crate::types::{GameState};

pub fn perft(depth: u8) -> u64 {
    let state = load_fen(STARTING_POSITION);
    perft_internal(&state, depth)
}

fn perft_internal(state: &GameState, depth: u8) -> u64 {
    let moves = legal_moves(state);

    if depth == 0 {
        return 1;
    } 

    return moves.iter().map(|m| {
        let mut new_state = state.clone();
        new_state.make_move(*m);
        perft_internal(&new_state, depth - 1)
    }).sum();
}

#[cfg(test)]
mod tests {
    use crate::perft::perft;

    #[test]
    fn perft_1() {
        assert_eq!(perft(1), 20);
    }

    #[test]
    fn perft_2() {
        assert_eq!(perft(2), 400);
    }

    #[test]
    fn perft_3() {
        assert_eq!(perft(3), 8902);
    }

    use rand::seq::SliceRandom;
    use crate::fen::{load_fen, STARTING_POSITION};
    use crate::moves::legal_moves;
    #[test]
    fn debugging() {
        let mut state = load_fen(STARTING_POSITION);

        for _ in 0..=3 {
            let moves = legal_moves(&state);
            let m = moves.choose(&mut rand::thread_rng());
            state.make_move(*m.unwrap());
            crate::board::print_board(&state.board);
            println!();
        }

        panic!("Fail the test");
    }
}
