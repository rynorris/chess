use std::fmt::Debug;
use chess_lib::types::ZobristHash;
use chess_lib::tt;

pub trait Game : Clone {
    type Move : Copy + Eq + Debug;

    fn make_move(&mut self, mv: Self::Move);
    fn legal_moves(&self) -> Vec<Self::Move>;
    fn zobrist_hash(&self) -> ZobristHash;
}

type Evaluator<T> = fn (&T) -> i64;

pub struct AlphaBeta<G : Game> {
    eval: Evaluator<G>,
    tt: tt::TranspositionTable<CacheData<G::Move>>,
}

#[derive(Clone, Copy)]
struct CacheData<M : Copy> {
    depth: u32,
    score: i64,
    best_move: Option<M>,
}

fn prefer_higher<M : Copy>(prev: CacheData<M>, new: CacheData<M>) -> tt::PolicyResult {
    if prev.depth < new.depth {
        tt::PolicyResult::Replace
    } else {
        tt::PolicyResult::Keep
    }
}

impl <G: Game> AlphaBeta<G> {
    pub fn new(eval: Evaluator<G>, tt_size: usize) -> AlphaBeta<G> {
        AlphaBeta{
            eval,
            tt: tt::TranspositionTable::new(tt_size, prefer_higher),
        }
    }

    pub fn tt_stats(&self) -> &tt::TTStats {
        self.tt.stats()
    }

    pub fn evaluate(&mut self, game: &G, depth: u32) -> (G::Move, i64) {
        for d in 0..=depth {
            self.eval_recursive(&game, d, i64::MIN + 1, i64::MAX - 1);
        }

        // Resconstruct the results from the TT.
        let root_data = self.tt.get(game.zobrist_hash()).expect("Root node not present in TT after evaluation");
        (root_data.best_move.unwrap(), root_data.score)
    }

    fn eval_recursive(
        &mut self,
        game: &G,
        depth: u32,
        mut alpha: i64,
        beta: i64,
    ) -> i64 {
        if depth == 0 {
            (self.eval)(game)
        } else {
            let zh = game.zobrist_hash();

            let cached_data = self.tt.get(zh);
            let cached_score = cached_data.and_then(|data| {
                if data.depth >= depth {
                    Some(data.score)
                } else {
                    None
                }
            });

            if cached_score.is_some() {
                return cached_score.unwrap();
            }

            let cached_best_move = cached_data.and_then(|data| data.best_move);
            let mut best_move: Option<G::Move> = None;
            let mut s = alpha;
            let moves = game.legal_moves();

            let best_move_first = cached_best_move.into_iter()
                .chain(moves.into_iter().filter(|m| cached_best_move != Some(*m)));

            for m in best_move_first {
                let mut new_state = game.clone();
                new_state.make_move(m);
                
                let eval = -self.eval_recursive(&new_state, depth - 1, -beta, -alpha);

                if eval >= beta {
                    s = beta;
                    best_move = Some(m);
                    break;
                }

                if eval > alpha {
                    alpha = eval;
                    s = eval;
                    best_move = Some(m);
                }
            }

            self.tt.insert(zh, CacheData{depth, score: s, best_move: best_move });
            s
        }
    }
}

