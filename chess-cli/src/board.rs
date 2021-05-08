use chess_lib::types::{Colour, GameState, Move, Piece};
use tui::buffer::Buffer;
use tui::layout::Rect;
use tui::style;
use tui::symbols::line;
use tui::widgets::{Block, Borders, Widget};

pub struct ChessBoard {
    state: GameState,
    highlight_move: Option<Move>,
}

impl ChessBoard {
    pub fn with_highlight(state: GameState, highlight: Move) -> ChessBoard {
        ChessBoard{ state, highlight_move: Some(highlight) }
    }
}

enum HBorder {
    None,
    Top,
    Middle,
    Bottom,
}

enum VBorder {
    None,
    Left,
    Middle,
    Right,
}

fn symbol_for_piece(piece: Piece) -> &'static str {
    match piece {
        Piece::King => "♚",
        Piece::Queen => "♛",
        Piece::Rook => "♜",
        Piece::Bishop => "♝",
        Piece::Knight => "♞",
        Piece::Pawn => "♟︎",
    }
}

impl Widget for ChessBoard {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default().title("Board").borders(Borders::ALL);
        block.render(area, buf);

        // Subtract 1 because there is one more border than squares.
        let square_width = ((area.width - 1) / 8) - 1;
        let square_height = ((area.height - 1) / 8) - 1;

        // 8 squares plus 9 borders.
        let board_width = square_width * 8 + 9;
        let board_height = square_height * 8 + 9;

        // Center the board.
        let board_x = area.x + (area.width - board_width) / 2;
        let board_y = area.y + (area.height - board_height) / 2;

        // Draw borders row by row.
        for x in 0..board_width {
            for y in 0..board_height {
                let h_border = if y == 0 {
                    HBorder::Top
                } else if y == board_height - 1{
                    HBorder::Bottom
                } else if y % (square_height + 1) == 0 {
                    HBorder::Middle
                } else {
                    HBorder::None
                };

                let v_border = if x == 0 {
                    VBorder::Left
                } else if x == board_width - 1 {
                    VBorder::Right
                } else if x % (square_width + 1) == 0 {
                    VBorder::Middle
                } else {
                    VBorder::None
                };

                let symbol = match (h_border, v_border) {
                    (HBorder::Top, VBorder::Left) => Some(line::THICK_TOP_LEFT),
                    (HBorder::Bottom, VBorder::Left) => Some(line::THICK_BOTTOM_LEFT),
                    (HBorder::Top, VBorder::Right) => Some(line::THICK_TOP_RIGHT),
                    (HBorder::Bottom, VBorder::Right) => Some(line::THICK_BOTTOM_RIGHT),
                    (HBorder::Top, VBorder::Middle) => Some(line::THICK_HORIZONTAL_DOWN),
                    (HBorder::Bottom, VBorder::Middle) => Some(line::THICK_HORIZONTAL_UP),
                    (HBorder::Middle, VBorder::Left) => Some(line::THICK_VERTICAL_RIGHT),
                    (HBorder::Middle, VBorder::Right) => Some(line::THICK_VERTICAL_LEFT),
                    (HBorder::Middle, VBorder::Middle) => Some(line::THICK_CROSS),
                    (HBorder::Top, VBorder::None) => Some(line::THICK_HORIZONTAL),
                    (HBorder::Bottom, VBorder::None) => Some(line::THICK_HORIZONTAL),
                    (HBorder::Middle, VBorder::None) => Some(line::THICK_HORIZONTAL),
                    (HBorder::None, VBorder::Left) => Some(line::THICK_VERTICAL),
                    (HBorder::None, VBorder::Right) => Some(line::THICK_VERTICAL),
                    (HBorder::None, VBorder::Middle) => Some(line::THICK_VERTICAL),
                    (_, _) => None,
                };

                match symbol {
                    Some(s) => {
                        buf.get_mut(board_x + x, board_y + y).set_symbol(s);
                    },
                    None => (),
                };
            }
        }

        // Draw pieces in the center of the squares.
        for file in 0..8u32 {
            for rank in 0..8u32 {
                match self.state.find_piece((file, rank).into()) {
                    Some((colour, piece)) => {
                        let symbol = symbol_for_piece(piece);
                        let colour = match colour {
                            Colour::White => style::Color::White,
                            Colour::Black => style::Color::Blue,
                        };
                        let x = (file as u16 * (square_width + 1)) + (square_width / 2) + 1;
                        let y = ((7 - rank) as u16 * (square_height + 1)) + (square_height / 2) + 1;
                        buf.get_mut(board_x + x, board_y + y).set_symbol(symbol).set_fg(colour);
                    },
                    None => (),
                }
            }
        }

        // Highlight the given move.
        match self.highlight_move {
            Some(Move::Normal(src, tgt)) => {
                let mut f = src.file() as u16;
                let mut r = src.rank() as u16;
                for x in (f * (square_width + 1) + 1)..((f + 1) * (square_width + 1)) {
                    for y in ((7 - r) * (square_height + 1) + 1)..((7 - r + 1) * (square_height + 1)) {
                        buf.get_mut(board_x + x, board_y + y).set_bg(style::Color::Yellow);
                    }
                }

                f = tgt.file() as u16;
                r = tgt.rank() as u16;
                for x in (f * (square_width + 1) + 1)..((f + 1) * (square_width + 1)) {
                    for y in ((7 - r) * (square_height + 1) + 1)..((7 - r + 1) * (square_height + 1)) {
                        buf.get_mut(board_x + x, board_y + y).set_bg(style::Color::Yellow);
                    }
                }
            },
            _ => (),
        };
    }
}
