use crate::types::{BitCoord, Colour, GameState, Move, Piece};

impl std::fmt::Display for GameState {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        format_board(self, formatter)?;
        Ok(())
    }
}

pub fn format_board(state: &GameState, formatter: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
    writeln!(formatter, " ------------------------------- ")?;
    for rank in (0..8u32).rev() {
        write!(formatter, "|")?;
        for file in 0..8u32 {
            let coord: BitCoord = (file, rank).into();
            write!(formatter, " ")?;
            match state.find_piece(coord) {
                Some((c, pc)) => {
                    match c {
                        Colour::White => write!(formatter, "{}", format_piece(pc))?,
                        Colour::Black => write!(formatter, "{}", format_piece(pc).to_lowercase())?,
                    }
                },
                None => write!(formatter, " ")?,
            }
            write!(formatter, " |")?;
        }
        writeln!(formatter)?;
        writeln!(formatter, " ------------------------------- ")?;
    }
    Ok(())
}

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

impl std::fmt::Debug for Move {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(formatter, "{}", format_move(*self))?;
        Ok(())
    }
}

impl std::fmt::Debug for BitCoord {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(formatter, "{}", format_coord(*self))?;
        Ok(())
    }
}

pub fn format_move(m: Move) -> String {
    match m {
        Move::Normal(_, src, tgt) => {
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

pub fn format_coord(coord: BitCoord) -> String {
     let mut s = String::new();
     s.push(format_file(coord.file()));
     s.push(format_rank(coord.rank()));
     s
}

pub fn format_file(file: u32) -> char {
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

pub fn format_rank(rank: u32) -> char {
    std::char::from_digit((rank + 1) as u32, 10).unwrap()
}

pub fn parse_coord(coord: &str) -> BitCoord {
    if coord.len() != 2 {
        panic!("Invalid coord: {}", coord);
    }

    let mut cs = coord.chars();
    let file = parse_file(cs.next().unwrap());
    let rank = parse_rank(cs.next().unwrap());

    (file, rank).into()
}

pub fn parse_file(c: char) -> u32 {
    match c {
        'a' => 0,
        'b' => 1,
        'c' => 2,
        'd' => 3,
        'e' => 4,
        'f' => 5,
        'g' => 6,
        'h' => 7,
        _ => panic!("Invalid file: {}", c),
    }
}

pub fn parse_rank(c: char) -> u32 {
    c.to_digit(10).expect(format!("Invalid rank: {}", c).as_str()) - 1
}

