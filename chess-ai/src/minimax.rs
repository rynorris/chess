use chess_lib::types::ZobristHash;
use chess_lib::tt;

pub trait Game : Clone {
    type Move : Copy;

    fn make_move(&mut self, mv: Self::Move);
    fn legal_moves(&self) -> Vec<Self::Move>;
    fn zobrist_hash(&self) -> ZobristHash;
}

type Evaluator<T> = fn (&T) -> i64;

pub struct AlphaBeta<G> {
    eval: Evaluator<G>,
    tt: tt::TranspositionTable<CacheData>,
}

#[derive(Clone, Copy)]
struct CacheData {
    depth: u32,
    score: i64,
}

fn prefer_higher(prev: CacheData, new: CacheData) -> tt::PolicyResult {
    if prev.depth <= new.depth {
        tt::PolicyResult::Keep
    } else {
        tt::PolicyResult::Replace
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

    pub fn evaluate(&mut self, game: &G, depth: u32) -> Vec<(G::Move, i64)> {
        game.legal_moves()
            .into_iter()
            .map(|m| {
                let mut new_state = game.clone();
                new_state.make_move(m);
                (m, -self.eval_recursive(&new_state, depth - 1, i64::MIN + 1, i64::MAX - 1))
            })
            .collect()
    }

    fn eval_recursive(
        &mut self,
        game: &G,
        depth: u32,
        mut alpha: i64,
        beta: i64,
    ) -> i64 {
        let zh = game.zobrist_hash();

        let score = match self.tt.get(zh) {
            Some(data) => data.score,
            None => {
                let mut s = alpha;

                if depth == 0 {
                    (self.eval)(game)
                } else {
                    for m in game.legal_moves().into_iter() {
                        let mut new_state = game.clone();
                        new_state.make_move(m);
                        
                        let eval = -self.eval_recursive(&new_state, depth - 1, -beta, -alpha);

                        if eval >= beta {
                            s = beta;
                            break;
                        }

                        if eval > alpha {
                            alpha = eval;
                            s = eval;
                        }
                    }

                    self.tt.insert(zh, CacheData{depth, score: s});
                    s
                }
            },
        };

        score
    }
}

