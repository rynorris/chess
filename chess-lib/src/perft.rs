
#[cfg(test)]
mod tests {
    #[test]
    fn perft_1() {
        let state = crate::fen::load_fen(crate::fen::STARTING_POSITION);
        let moves = crate::moves::legal_moves(&state);
        println!("{:?}", moves);
        assert_eq!(moves.len(), 20);
    }
}
