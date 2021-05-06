#[macro_use]
pub mod board;
pub mod fen;
pub mod fmt;
pub mod game;
pub mod magic;
pub mod moves;
pub mod perft;
pub mod pgn;
pub mod types;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
