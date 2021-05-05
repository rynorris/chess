use crate::board::{Coords, empty_board};
use crate::types::{Board, GameState, Colour, Coordinate, Piece, SideState, Square};

pub const STARTING_POSITION: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

pub fn load_fen(fen: &str) -> GameState {
    let mut fields = fen.split_whitespace();

    let positions = fields.next().expect("FEN string didn't contain piece positions");
    let mut board = empty_board();
    // FEN starts at a8.
    let mut coord = 0x07;
    for c in positions.chars() {
        // Just do this naively.
        match c {
            'Q' => board[coord as usize] = Square::Occupied(Colour::White, Piece::Queen),
            'K' => board[coord as usize] = Square::Occupied(Colour::White, Piece::King),
            'R' => board[coord as usize] = Square::Occupied(Colour::White, Piece::Rook),
            'B' => board[coord as usize] = Square::Occupied(Colour::White, Piece::Bishop),
            'N' => board[coord as usize] = Square::Occupied(Colour::White, Piece::Knight),
            'P' => board[coord as usize] = Square::Occupied(Colour::White, Piece::Pawn),
            'q' => board[coord as usize] = Square::Occupied(Colour::Black, Piece::Queen),
            'k' => board[coord as usize] = Square::Occupied(Colour::Black, Piece::King),
            'r' => board[coord as usize] = Square::Occupied(Colour::Black, Piece::Rook),
            'b' => board[coord as usize] = Square::Occupied(Colour::Black, Piece::Bishop),
            'n' => board[coord as usize] = Square::Occupied(Colour::Black, Piece::Knight),
            'p' => board[coord as usize] = Square::Occupied(Colour::Black, Piece::Pawn),
            '/' => {
                coord = (coord & 0x0F) - 0x01;
                continue;
            },  // Next rank down.
            '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' => {
                let num_empty = c.to_digit(10).unwrap();
                coord += num_empty * 0x10;
                continue
            },
            _ => panic!("Unexpected character in piece positions: {}", c),
        }

        coord += 0x10;
    }

    // Initialize side states from board.
    let mut white = SideState{
        king_coord: expect_king(&board, Colour::White),
        can_castle_kingside: false,
        can_castle_queenside: false,
    };

    let mut black = SideState{
        king_coord: expect_king(&board, Colour::Black),
        can_castle_kingside: false,
        can_castle_queenside: false,
    };

    let active_colour_field = fields.next().expect("FEN string didn't contain active colour");
    let active_colour = match active_colour_field {
        "w" => Colour::White,
        "b" => Colour::Black,
        _ => panic!("Invalid active colour field: {}", active_colour_field),
    };

    let castling_field = fields.next().expect("FEN string didn't contain castling");
    for c in castling_field.chars() {
        match c {
            'K' => white.can_castle_kingside = true,
            'Q' => white.can_castle_queenside = true,
            'k' => black.can_castle_kingside = true,
            'q' => black.can_castle_queenside = true,
            '-' => (),
            _ => panic!("Invalid castling field: {}", castling_field),
        }
    }

    GameState{
        board,
        active_colour,
        white,
        black,
        en_passant: None,
        fifty_move_clock: 0,
    }
}

fn expect_king(board: &Board, colour: Colour) -> Coordinate {
    let king_coords: Vec<Coordinate> = Coords::new().filter(|c| match board[*c as usize] {
        Square::Occupied(col, Piece::King) => col == colour,
        _ => false,
    }).collect();

    if king_coords.len() == 1 {
        *king_coords.iter().next().unwrap()
    } else {
        panic!("Didn't find single king");
    }
}

#[cfg(test)]
mod tests {
    use crate::fen::*;

    #[test]
    fn starting_position() {
        let state = load_fen(STARTING_POSITION);

        // Check pieces rank by rank.
        assert_eq!(state.board[0x07], Square::Occupied(Colour::Black, Piece::Rook));
        assert_eq!(state.board[0x17], Square::Occupied(Colour::Black, Piece::Knight));
        assert_eq!(state.board[0x27], Square::Occupied(Colour::Black, Piece::Bishop));
        assert_eq!(state.board[0x37], Square::Occupied(Colour::Black, Piece::Queen));
        assert_eq!(state.board[0x47], Square::Occupied(Colour::Black, Piece::King));
        assert_eq!(state.board[0x57], Square::Occupied(Colour::Black, Piece::Bishop));
        assert_eq!(state.board[0x67], Square::Occupied(Colour::Black, Piece::Knight));
        assert_eq!(state.board[0x77], Square::Occupied(Colour::Black, Piece::Rook));

        for file in 0..=7 {
            assert_eq!(state.board[0x06 | (file << 4)], Square::Occupied(Colour::Black, Piece::Pawn));
        }

        for rank in 2..=5 {
            for file in 0..=7 {
                assert_eq!(state.board[(file << 4) | rank], Square::Empty);
            }
        }

        for file in 0..=7 {
            assert_eq!(state.board[0x01 | (file << 4)], Square::Occupied(Colour::White, Piece::Pawn));
        }

        assert_eq!(state.board[0x00], Square::Occupied(Colour::White, Piece::Rook));
        assert_eq!(state.board[0x10], Square::Occupied(Colour::White, Piece::Knight));
        assert_eq!(state.board[0x20], Square::Occupied(Colour::White, Piece::Bishop));
        assert_eq!(state.board[0x30], Square::Occupied(Colour::White, Piece::Queen));
        assert_eq!(state.board[0x40], Square::Occupied(Colour::White, Piece::King));
        assert_eq!(state.board[0x50], Square::Occupied(Colour::White, Piece::Bishop));
        assert_eq!(state.board[0x60], Square::Occupied(Colour::White, Piece::Knight));
        assert_eq!(state.board[0x70], Square::Occupied(Colour::White, Piece::Rook));

        assert_eq!(state.white.king_coord, 0x40);
        assert_eq!(state.black.king_coord, 0x47);

        assert_eq!(state.white.can_castle_kingside, true);
        assert_eq!(state.white.can_castle_queenside, true);
        assert_eq!(state.black.can_castle_kingside, true);
        assert_eq!(state.black.can_castle_queenside, true);
    }
}
