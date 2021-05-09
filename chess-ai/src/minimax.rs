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
    score: i64,
}

impl <G: Game> AlphaBeta<G> {
    pub fn new(eval: Evaluator<G>) -> AlphaBeta<G> {
        AlphaBeta{
            eval,
            tt: tt::TranspositionTable::new(1 << 25, tt::always_replace),
        }
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
        if depth == 0 {
            return (self.eval)(game);
        }

        for m in game.legal_moves().into_iter() {
            let mut new_state = game.clone();
            new_state.make_move(m);
            
            let zh = new_state.zobrist_hash();
            let score = match self.tt.get(zh) {
                Some(data) => data.score,
                None => {
                    let s = -self.eval_recursive(&new_state, depth - 1, -beta, -alpha);
                    self.tt.insert(zh, CacheData{score: s});
                    s
                },
            };

            if score >= beta {
                return beta;
            }
            if score > alpha {
                alpha = score;
            }
        }

        return alpha;
    }
}

