use std::collections::HashMap;
use crate::fmt::{format_move};
use crate::magic::MagicBitBoards;
use crate::moves::legal_moves;
use crate::types::{GameState};

pub fn perft(state: &GameState, depth: u8, mbb: &MagicBitBoards) -> u64 {
    if depth == 0 {
        return 1;
    } 

    let moves = legal_moves(state, &mbb);

    return moves.iter().map(|m| {
        let mut new_state = state.clone();
        new_state.make_move(*m);
        perft(&new_state, depth - 1, mbb)
    }).sum();
}

pub fn divide(state: &GameState, depth: u8, mbb: &MagicBitBoards) -> HashMap<String, u64> {
    if depth < 1 {
        panic!("Divide requires depth at least 1");
    }

    let moves = legal_moves(state, &mbb);

    let results: Vec<(String, u64)> = moves.iter().map(|m| {
        let move_str = format_move(*m);
        let mut state_2 = state.clone();
        state_2.make_move(*m);
        (move_str, perft(&state_2, depth - 1, mbb))
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
    use crate::magic::MagicBitBoards;
    use crate::perft::{perft};

    macro_rules! perft_test {
        [ $name:ident: Starting at position $position:expr, at depth $depth:expr, the number of possible moves is: $count:expr ] => {
            #[test]
            fn $name() {
                let state = load_fen($position);
                let mbb = MagicBitBoards::default();
                assert_eq!(perft(&state, $depth, &mbb), $count);
            }
        };
    }

    // Example positions and results taken from https://www.chessprogramming.org/Perft_Results
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

    perft_test![ starting_5:
        Starting at position STARTING_POSITION,
        at depth 5, the number of possible moves is: 4_865_609
    ];

    perft_test![ position_2_1:
        Starting at position "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - ",
        at depth 1, the number of possible moves is: 48
    ];

    perft_test![ position_2_2:
        Starting at position "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - ",
        at depth 2, the number of possible moves is: 2_039
    ];

    perft_test![ position_2_3:
        Starting at position "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - ",
        at depth 3, the number of possible moves is: 97_862
    ];

    perft_test![ position_3_1:
        Starting at position "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - ",
        at depth 1, the number of possible moves is: 14
    ];

    perft_test![ position_3_2:
        Starting at position "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - ",
        at depth 2, the number of possible moves is: 191
    ];

    perft_test![ position_3_3:
        Starting at position "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - ",
        at depth 3, the number of possible moves is: 2_812
    ];

    perft_test![ position_3_4:
        Starting at position "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - ",
        at depth 4, the number of possible moves is: 43_238
    ];

    perft_test![ position_3_5:
        Starting at position "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - ",
        at depth 5, the number of possible moves is: 674_624
    ];

    perft_test![ position_4_1:
        Starting at position "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1",
        at depth 1, the number of possible moves is: 6
    ];

    perft_test![ position_4_2:
        Starting at position "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1",
        at depth 2, the number of possible moves is: 264
    ];

    perft_test![ position_4_3:
        Starting at position "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1",
        at depth 3, the number of possible moves is: 9_467
    ];

    perft_test![ position_4_4:
        Starting at position "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1",
        at depth 4, the number of possible moves is: 422_333
    ];

    perft_test![ position_5_1:
        Starting at position "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8",
        at depth 1, the number of possible moves is: 44
    ];

    perft_test![ position_5_2:
        Starting at position "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8",
        at depth 2, the number of possible moves is: 1_486
    ];

    perft_test![ position_5_3:
        Starting at position "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8",
        at depth 3, the number of possible moves is: 62_379
    ];

    perft_test![ position_6_1:
        Starting at position "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10 ",
        at depth 1, the number of possible moves is: 46
    ];

    perft_test![ position_6_2:
        Starting at position "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10 ",
        at depth 2, the number of possible moves is: 2_079
    ];

    perft_test![ position_6_3:
        Starting at position "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10 ",
        at depth 3, the number of possible moves is: 89_890
    ];

    // This position stress tests promotion bugs.
    // Taken from http://www.rocechess.ch/perft.html
    perft_test![ promotions_1:
        Starting at position "n1n5/PPPk4/8/8/8/8/4Kppp/5N1N b - - 0 1",
        at depth 1, the number of possible moves is: 24
    ];

    perft_test![ promotions_2:
        Starting at position "n1n5/PPPk4/8/8/8/8/4Kppp/5N1N b - - 0 1",
        at depth 2, the number of possible moves is: 496
    ];

    perft_test![ promotions_3:
        Starting at position "n1n5/PPPk4/8/8/8/8/4Kppp/5N1N b - - 0 1",
        at depth 3, the number of possible moves is: 9_483
    ];

    /*
    perft_test![ position_5_4:
        Starting at position "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8",
        at depth 4, the number of possible moves is: 2_103_487
    ];
    */
}
