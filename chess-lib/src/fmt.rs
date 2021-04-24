use crate::types::{Coordinate, Move, Piece};
use crate::board::{file, rank};

pub fn format_piece(piece: Piece) -> char {
    match piece {
        Piece::King => 'K',
        Piece::Queen => 'Q',
        Piece::Rook => 'R',
        Piece::Bishop => 'B',
        Piece::Knight => 'N',
        Piece::Pawn => 'P',
    }
}

pub fn format_move(m: Move) -> String {
    match m {
        Move::Normal(src, tgt) => {
            let s = format!("{}{}", format_coord(src), format_coord(tgt));
            s
        },
        Move::Promotion(src, tgt, pc) => {
            let s = format!("{}{}{}", format_coord(src), format_coord(tgt), format_piece(pc));
            s
        },
        Move::Castle => "O-O".to_owned(),
        Move::LongCastle => "O-O-O".to_owned(),
    }
}

pub fn format_coord(coord: Coordinate) -> String {
     let mut s = String::new();
     s.push(format_file(file(coord)));
     s.push(format_rank(rank(coord)));
     s
}

pub fn format_file(file: u8) -> char {
    match file {
        0 => 'a',
        1 => 'b',
        2 => 'c',
        3 => 'd',
        4 => 'e',
        5 => 'f',
        6 => 'g',
        7 => 'h',
        _ => panic!("Invalid file: {}", file),
    }
}

pub fn format_rank(rank: u8) -> char {
    std::char::from_digit((rank + 1) as u32, 10).unwrap()
}

