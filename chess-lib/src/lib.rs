#[macro_use]
pub mod board;
pub mod fen;
pub mod moves;
pub mod perft;
pub mod types;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
