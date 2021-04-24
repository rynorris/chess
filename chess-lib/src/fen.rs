use std::collections::HashSet;
use crate::board::{coord, empty_board};
use crate::types::{Board, GameState, Colour, Coordinate, Piece, SideState, Square};

pub const STARTING_POSITION: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

pub fn load_fen(fen: &str) -> GameState {
    let mut fields = fen.split_whitespace();

    let positions = fields.next().expect("FEN string didn't contain piece positions");
    let mut board = empty_board();
    // FEN starts at a8.
    let mut coord = 0x07;
    for c in positions.chars() {
        println!("c = {}, coord = {:x}", c, coord);

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
        piece_coords: piece_coords(&board, Colour::White),
        king_coord: expect_king(&board, Colour::White),
        can_castle_kingside: false,
        can_castle_queenside: false,
    };

    let mut black = SideState{
        piece_coords: piece_coords(&board, Colour::Black),
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
    }
}

fn expect_king(board: &Board, colour: Colour) -> Coordinate {
    let king_coords = find_pieces(board, colour, Piece::King);
    if king_coords.len() == 1 {
        *king_coords.iter().next().unwrap()
    } else {
        panic!("Didn't find single king");
    }
}

fn find_pieces(board: &Board, colour: Colour, piece: Piece) -> HashSet<Coordinate> {
    let mut coords = HashSet::new();
    for file in 0..=7 {
        for rank in 0..=7 {
            match board[coord(file, rank) as usize] {
                Square::Occupied(col, p) => {
                    if col == colour && p == piece {
                        coords.insert(coord(file, rank));
                    }
                },
                Square::Empty => (),
            }
        }
    }
    coords
}

fn piece_coords(board: &Board, colour: Colour) -> HashSet<Coordinate> {
    let mut coords = HashSet::new();
    for file in 0..=7 {
        for rank in 0..=7 {
            match board[coord(file, rank) as usize] {
                Square::Occupied(col, _) => {
                    if col == colour {
                        coords.insert(coord(file, rank));
                    }
                },
                Square::Empty => (),
            }
        }
    }
    coords
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

        assert_eq!(state.white.piece_coords, vec![
            0x00, 0x10, 0x20, 0x30, 0x40, 0x50, 0x60, 0x70,
            0x01, 0x11, 0x21, 0x31, 0x41, 0x51, 0x61, 0x71,
        ].into_iter().collect::<HashSet<Coordinate>>());

        assert_eq!(state.black.piece_coords, vec![
            0x07, 0x17, 0x27, 0x37, 0x47, 0x57, 0x67, 0x77,
            0x06, 0x16, 0x26, 0x36, 0x46, 0x56, 0x66, 0x76,
        ].into_iter().collect::<HashSet<Coordinate>>());

        assert_eq!(state.white.king_coord, 0x40);
        assert_eq!(state.black.king_coord, 0x47);

        assert_eq!(state.white.can_castle_kingside, true);
        assert_eq!(state.white.can_castle_queenside, true);
        assert_eq!(state.black.can_castle_kingside, true);
        assert_eq!(state.black.can_castle_queenside, true);
    }
}
