use std::collections::HashMap;
use crate::fmt::{format_move};
use crate::moves::legal_moves;
use crate::types::{GameState};

pub fn perft(state: &GameState, depth: u8) -> u64 {
    if depth == 0 {
        return 1;
    } 

    let moves = legal_moves(state);

    return moves.iter().map(|m| {
        let mut new_state = state.clone();
        new_state.make_move(*m);
        perft(&new_state, depth - 1)
    }).sum();
}

pub fn divide(state: &GameState, depth: u8) -> HashMap<String, u64> {
    if depth < 1 {
        panic!("Divide requires depth at least 1");
    }

    let moves = legal_moves(state);

    let results: Vec<(String, u64)> = moves.iter().map(|m| {
        let move_str = format_move(*m);
        let mut state_2 = state.clone();
        state_2.make_move(*m);
        (move_str, perft(&state_2, depth - 1))
    }).collect();
    
    let mut counts: HashMap<String, u64> = HashMap::new();
    results.iter().for_each(|(m, c)| {
        counts.insert(m.clone(), *c);
    });

    counts
}

#[cfg(test)]
mod tests {
    use crate::fen::{load_fen, STARTING_POSITION};
    use crate::perft::{perft};

    macro_rules! perft_test {
        [ $name:ident: Starting at position $position:expr, at depth $depth:expr, the number of possible moves is: $count:expr ] => {
            #[test]
            fn $name() {
                let state = load_fen($position);
                assert_eq!(perft(&state, $depth), $count);
            }
        };
    }

    perft_test![ starting_1:
        Starting at position STARTING_POSITION,
        at depth 1, the number of possible moves is: 20
    ];
    
    perft_test![ starting_2:
        Starting at position STARTING_POSITION,
        at depth 2, the number of possible moves is: 400
    ];

    perft_test![ starting_3:
        Starting at position STARTING_POSITION,
        at depth 3, the number of possible moves is: 8902
    ];

    perft_test![ starting_4:
        Starting at position STARTING_POSITION,
        at depth 4, the number of possible moves is: 197_281
    ];

    /*
    use rand::seq::SliceRandom;
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

    #[test]
    fn divide_3() {
        let state = load_fen("rnbqkbnr/ppp1pppp/8/3N4/8/8/PPPPPPPP/R1BQKBNR b KQkq - 0 1");
        //let mut state = load_fen(STARTING_POSITION);
        //state.make_move(crate::types::Move::Normal(0x10, 0x22));
        //state.make_move(crate::types::Move::Normal(0x36, 0x35));
        crate::board::print_board(&state.board);
        let counts = divide(&state, 1);
        let mut lines: Vec<String> = counts.iter().map(|(k, v)| {
            return format!("{}: {}", k, v);
        }).collect();
        lines.sort();
        lines.iter().for_each(|l| println!("{}", l));
        panic!("Fail the test");
    }
    */
}
