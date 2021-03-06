#[macro_use]
pub mod fen;
pub mod fmt;
pub mod game;
pub mod magic;
pub mod moves;
pub mod perft;
pub mod pgn;
pub mod tt;
pub mod types;
pub mod zobrist;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
