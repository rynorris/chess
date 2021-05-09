use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use chess_lib::fen::{load_fen, STARTING_POSITION};
use chess_lib::magic::MagicBitBoards;
use chess_lib::moves::legal_moves;
use chess_lib::types::Move;
use chess_lib::zobrist::ZobristHasher;

#[test]
fn zobrist_chaos() {
    // Tests that incrementally updated zobrist hash equals one computed from scratch.
    let mut rng = ChaCha8Rng::seed_from_u64(12345);
    let mbb = MagicBitBoards::default();
    let hasher = ZobristHasher::default();
    let mut state = load_fen(STARTING_POSITION);
    let mut sequence: Vec<Move> = vec![];

    for _ in 0..100_000 {
        let moves = legal_moves(&state, &mbb);
        match moves.choose(&mut rng) {
            Some(mv) => {
                state.make_move(*mv);
                sequence.push(*mv);
                let clean_hash = hasher.hash(&state);

                if state.zh != clean_hash {
                    println!("After these moves, the hash differs: {:?}", sequence);
                    println!("Final board: \n{}", &state);
                    println!("Hash difference: 0x{:16x}", state.zh.0 ^ clean_hash.0);
                    println!("Identified error index: {:?}", hasher.identify_diff(state.zh, clean_hash));
                    panic!("Test failed");
                }
            },
            None => {
                // No legal moves, reset game.
                state = load_fen(STARTING_POSITION);
                sequence.clear();
            },
        }

        // Also reset after 50 moves.
        if sequence.len() >= 50 {
            state = load_fen(STARTING_POSITION);
            sequence.clear();
        }
    }
}
