use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::rc::Rc;

use rand::seq::SliceRandom;

pub struct Node<Move> {
    wins: f32,
    simulations: f32,
    children: HashMap<Move, Rc<RefCell<Node<Move>>>>,
    parent: Option<Rc<RefCell<Node<Move>>>>,
    mv: Option<Move>,
}

impl <Move : Hash + Eq> Node<Move> {
    fn root() -> Node<Move> {
        Node{
            wins: 0.0,
            simulations: 0.0,
            children: HashMap::new(),
            parent: None,
            mv: None,
        }
    }

    fn child_of(parent: Rc<RefCell<Node<Move>>>, mv: Move) -> Node<Move> {
        Node{
            wins: 0.0,
            simulations: 0.0,
            children: HashMap::new(),
            parent: Some(parent),
            mv: Some(mv),
        }
    }
}

pub trait Game : Clone {
    type Move;

    fn make_move(&mut self, mv: Self::Move);
    fn legal_moves(&self) -> Vec<Self::Move>;
    fn game_state(&self) -> GameState;
    fn active_player(&self) -> Player;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GameState {
    Ongoing,
    Finished(GameResult),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Player {
    One,
    Two,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GameResult {
    Win,
    Loss,
    Draw,
}

impl GameResult {
    pub fn reverse(res: GameResult) -> GameResult {
        match res {
            GameResult::Win => GameResult::Loss,
            GameResult::Loss => GameResult::Win,
            GameResult::Draw => GameResult::Draw,
        }
    }
}

pub struct MCTS<M, G> {
    root: Rc<RefCell<Node<M>>>,
    initial_state: G,
}

impl <M : Hash + Eq + Copy + Debug, G : Game<Move = M>> MCTS<M, G> {
    pub fn new(initial_state: G) -> MCTS<M, G> {
        MCTS{
            root: Rc::new(RefCell::new(Node::root())),
            initial_state,
        }
    }

    pub fn best_move(&self) -> M {
        let root = self.root.borrow();
        let (_, best) = root.children.iter()
            .fold(
                (None, None),
                |acc, nd| {
                    match acc {
                        (Some(max), _) => if nd.1.borrow().simulations > max {
                            (Some(nd.1.borrow().simulations), Some(nd.0))
                        } else {
                            acc
                        },
                        (None, _) => (Some(nd.1.borrow().simulations), Some(nd.0)),
                    }
                }
            );

        *best.unwrap()
    }

    pub fn move_scores(&self) -> Vec<(M, f32, f32)> {
        self.root.borrow().children.iter().map(|(mv, nd)| (*mv, nd.borrow().wins, nd.borrow().simulations)).collect()
    }

    pub fn simulate_once(&mut self) {
        let (leaf, state) = MCTS::traverse(self.root.clone(), self.initial_state.clone());
        let result = MCTS::rollout(state);
        Self::back_propagate(leaf, result);
    }

    pub fn traverse(node: Rc<RefCell<Node<M>>>, state: G) -> (Rc<RefCell<Node<M>>>, G) {
        let legal_moves = state.legal_moves();
        let unexplored_moves: Vec<M> = legal_moves.into_iter().filter(|m| !node.borrow().children.contains_key(&m)).collect();
        let mut new_state = state.clone();

        if unexplored_moves.len() > 0 {
            let mv = unexplored_moves.choose(&mut rand::thread_rng()).unwrap();
            new_state.make_move(*mv);
            let nd = Rc::new(RefCell::new(Node::child_of(node.clone(), *mv)));
            node.borrow_mut().children.insert(*mv, nd.clone());
            return (nd, new_state);
        } else {
            // No new moves to expand, continue down the tree, or return this node if the game is
            // over.
            match Self::select_child_uct(node.clone()) {
                Some(child) => {
                    new_state.make_move(child.borrow().mv.unwrap());
                    Self::traverse(child, new_state)
                },
                None => (node, state),
            }
        }
    }

    fn select_child_uct(node: Rc<RefCell<Node<M>>>) -> Option<Rc<RefCell<Node<M>>>> {
        let parent_simulations = node.borrow().simulations;
        let (_, best) = node.borrow().children.iter()
            .fold(
                (None, None),
                |acc, nd| {
                    let uct = Self::uct_formula(nd.1.borrow().wins, nd.1.borrow().simulations, parent_simulations);
                    match acc {
                        (Some(max), _) => if uct > max {
                            (Some(uct), Some(nd.1.clone()))
                        } else {
                            acc
                        },
                        (None, _) => (Some(uct), Some(nd.1.clone())),
                    }
                }
            );

        best
    }

    fn uct_formula(wins: f32, simulations: f32, parent_simulations: f32) -> f32 {
        let exploitation = wins / simulations;
        let c = 2f32.sqrt();
        let exploration = c * (parent_simulations.ln() / simulations).sqrt();
        exploitation + exploration
    }

    fn rollout(state: G) -> GameResult {
        let mut rollout_state = state.clone();
        let this_player = state.active_player();

        let mut game_state = state.game_state();
        while game_state == GameState::Ongoing {
            let legal_moves = rollout_state.legal_moves();
            let mv = legal_moves.choose(&mut rand::thread_rng()).unwrap();
            rollout_state.make_move(*mv);
            game_state = rollout_state.game_state();
        }

        let mut result = match game_state {
            GameState::Ongoing => panic!("Unreachable branch"),
            GameState::Finished(res) => res,
        };

        let final_player = rollout_state.active_player();
        if final_player == this_player {
            result = GameResult::reverse(result);
        }

        result
    }

    fn back_propagate(node: Rc<RefCell<Node<M>>>, result: GameResult) {
        let mut nd = node.borrow_mut();
        nd.simulations += 1.0;
        if result == GameResult::Win {
            nd.wins += 1.0;
        } else if result == GameResult::Draw {
            nd.wins += 0.5;
        }

        match nd.parent.clone() {
            Some(p) => Self::back_propagate(p, GameResult::reverse(result)),
            None => (),
        };
    }
}

