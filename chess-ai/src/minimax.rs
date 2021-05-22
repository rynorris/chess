use chess_lib::types::ZobristHash;
use chess_lib::tt;

pub trait Game : Clone {
    type Move : Copy + Eq;

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
        let mut result = vec![];
        for d in 0..depth {
            result = game.legal_moves()
                .into_iter()
                .map(|m| {
                    let mut new_state = game.clone();
                    new_state.make_move(m);
                    (m, -self.eval_recursive(&new_state, d, i64::MIN + 1, i64::MAX - 1))
                })
                .collect();
        }
        result
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

            let mut best_move = self.tt.get(zh).and_then(|data| data.best_move);
            let mut s = alpha;
            let mut moves = game.legal_moves();
            moves.sort_unstable_by_key(|m| if best_move == Some(*m) { 0 } else { 1 });

            for m in moves.into_iter() {
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

