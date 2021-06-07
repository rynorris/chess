use chess_lib::types::{BitBoard, Colour, Pieces};
use crate::chess::Chess;

// Score in centipawns.
pub fn evaluate(chess: &Chess) -> i64 {
    let (active_side, other_side) = match chess.state.active_colour {
        Colour::White => (&chess.state.white, &chess.state.black),
        Colour::Black => (&chess.state.black, &chess.state.white),
    };

    count_material(&active_side.pieces) - count_material(&other_side.pieces)
}

fn count_material(pieces: &Pieces) -> i64 {
    let mut material: i64 = 0;

    material += count_piece(pieces.queens, 900);
    material += count_piece(pieces.rooks, 500);
    material += count_piece(pieces.bishops, 300);
    material += count_piece(pieces.knights, 300);
    material += count_piece(pieces.pawns, 100);

    material
}

fn count_piece(bb: BitBoard, value: i64) -> i64 {
    (bb.count() as i64) * value
}
