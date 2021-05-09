use crate::fmt::parse_coord;
use crate::types::{BitCoord, GameState, Colour, Piece, Pieces, SideState};

pub const STARTING_POSITION: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

pub fn load_fen(fen: &str) -> GameState {
    let mut fields = fen.split_whitespace();

    let positions = fields.next().expect("FEN string didn't contain piece positions");

    let mut w_pieces = Pieces::empty();
    let mut b_pieces = Pieces::empty();

    // FEN starts at a8.
    let mut coord = BitCoord(0x80_00_00_00_00_00_00_00);
    for c in positions.chars() {
        // Just do this naively.
        match c {
            'Q' => w_pieces.put_piece(Piece::Queen, coord),
            'K' => w_pieces.put_piece(Piece::King, coord),
            'R' => w_pieces.put_piece(Piece::Rook, coord),
            'B' => w_pieces.put_piece(Piece::Bishop, coord),
            'N' => w_pieces.put_piece(Piece::Knight, coord),
            'P' => w_pieces.put_piece(Piece::Pawn, coord),
            'q' => b_pieces.put_piece(Piece::Queen, coord),
            'k' => b_pieces.put_piece(Piece::King, coord),
            'r' => b_pieces.put_piece(Piece::Rook, coord),
            'b' => b_pieces.put_piece(Piece::Bishop, coord),
            'n' => b_pieces.put_piece(Piece::Knight, coord),
            'p' => b_pieces.put_piece(Piece::Pawn, coord),
            '/' => {
                continue;
            },  // Next rank down.
            '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' => {
                let num_empty = c.to_digit(10).unwrap();
                coord = coord >> num_empty;
                continue
            },
            _ => panic!("Unexpected character in piece positions: {}", c),
        }

        coord = coord >> 1;
    }

    // Initialize side states from board.
    let mut white = SideState{
        pieces: w_pieces,
        can_castle_kingside: false,
        can_castle_queenside: false,
    };

    let mut black = SideState{
        pieces: b_pieces,
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

    let en_passant_field = fields.next().expect("FEN string didn't contain en-passant");
    let en_passant = match en_passant_field {
        "-" => None,
        _ => Some(parse_coord(en_passant_field)),
    };

    GameState::new(
        active_colour,
        white,
        black,
        en_passant,
        0,
    )
}

#[cfg(test)]
mod tests {
    use crate::fen::*;
    use crate::types::BitBoard;

    #[test]
    fn starting_position() {
        let state = load_fen(STARTING_POSITION);

        // Check pieces.
        assert_eq!(state.white.pieces.king, BitBoard(0x00_00_00_00_00_00_00_08));
        assert_eq!(state.white.pieces.queens, BitBoard(0x00_00_00_00_00_00_00_10));
        assert_eq!(state.white.pieces.rooks, BitBoard(0x00_00_00_00_00_00_00_81));
        assert_eq!(state.white.pieces.bishops, BitBoard(0x00_00_00_00_00_00_00_24));
        assert_eq!(state.white.pieces.knights, BitBoard(0x00_00_00_00_00_00_00_42));
        assert_eq!(state.white.pieces.pawns, BitBoard(0x00_00_00_00_00_00_FF_00));

        assert_eq!(state.black.pieces.king, BitBoard(0x08_00_00_00_00_00_00_00));
        assert_eq!(state.black.pieces.queens, BitBoard(0x10_00_00_00_00_00_00_00));
        assert_eq!(state.black.pieces.rooks, BitBoard(0x81_00_00_00_00_00_00_00));
        assert_eq!(state.black.pieces.bishops, BitBoard(0x24_00_00_00_00_00_00_00));
        assert_eq!(state.black.pieces.knights, BitBoard(0x42_00_00_00_00_00_00_00));
        assert_eq!(state.black.pieces.pawns, BitBoard(0x00_FF_00_00_00_00_00_00));

        assert_eq!(state.white.can_castle_kingside, true);
        assert_eq!(state.white.can_castle_queenside, true);
        assert_eq!(state.black.can_castle_kingside, true);
        assert_eq!(state.black.can_castle_queenside, true);
    }
}
