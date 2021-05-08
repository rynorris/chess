use std::io;
use std::time::Instant;
use clap::{AppSettings, Clap};
use termion::event::{Key};
use termion::input::{TermRead};
use termion::raw::IntoRawMode;
use tui::Terminal;
use tui::backend::TermionBackend;
use tui::widgets::{BarChart, Block, Borders};
use tui::layout::{Layout, Constraint, Direction};
use tui::style::{Color, Style};

mod board;

#[derive(Clap)]
#[clap(version = "0.1", author = "Ryan N. <rynorris@gmail.com>")]
#[clap(setting = AppSettings::ColoredHelp)]
struct Opts {
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Clap)]
enum SubCommand {
    Analyze(Analyze),
    Divide(Divide),
    Magic(Magic),
}

#[derive(Clap)]
struct Divide {
    #[clap(short)]
    fen: String,

    #[clap(short)]
    depth: u8,
}

#[derive(Clap)]
struct Analyze {
    #[clap(short)]
    engine: String,

    #[clap(short)]
    fen: String,

    #[clap(short)]
    simulations: u64,
}

#[derive(Clap)]
struct Magic {
    #[clap(short)]
    piece: String,
}

fn main() -> Result<(), io::Error> {
    let opts: Opts = Opts::parse();
    match opts.subcmd {
        SubCommand::Divide(div) => {
            let state = chess_lib::fen::load_fen(&div.fen);
            let mbb = chess_lib::magic::MagicBitBoards::default();

            let before = Instant::now();
            let counts = chess_lib::perft::divide(&state, div.depth, &mbb);
            let after = Instant::now();

            let mut lines: Vec<String> = counts.iter().map(|(k, v)| {
                return format!("{}: {}", k, v);
            }).collect();
            lines.sort();
            lines.iter().for_each(|l| println!("{}", l));
            println!("Total nodes: {}", counts.iter().map(|(_, v)| *v).sum::<u64>());

            let duration = after - before;
            println!("Took: {}s", duration.as_secs_f32());
            Ok(())
        },
        SubCommand::Analyze(cmd) => {
            match cmd.engine.as_str() {
                "minimax" => {
                    let state = chess_lib::fen::load_fen(&cmd.fen);
                    let mbb = chess_lib::magic::MagicBitBoards::default();
                    let chess = chess_ai::chess::Chess::new(state, &mbb);
                    let alphabeta = chess_ai::minimax::AlphaBeta::new(chess_ai::eval::evaluate);

                    let mut results = alphabeta.evaluate(&chess, cmd.simulations as u32);
                    results.sort_by(|(_, a), (_, b)| b.partial_cmp(a).unwrap());
                    results.into_iter().for_each(|(m, s)| {
                        println!("{}: {}", chess_lib::fmt::format_move(m), s);
                    });

                    Ok(())
                },
                "montecarlo" => {
                    let stdout = io::stdout().into_raw_mode()?;
                    let mut stdin = termion::async_stdin().keys();
                    let backend = TermionBackend::new(stdout);
                    let mut terminal = Terminal::new(backend)?;
                    terminal.clear()?;
                    let state = chess_lib::fen::load_fen(&cmd.fen);
                    let draw_state = state.clone();

                    let mbb = chess_lib::magic::MagicBitBoards::default();
                    let chess = chess_ai::chess::Chess::new(state, &mbb);
                    let mut monte = chess_ai::montecarlo::MCTS::new(chess);

                    loop {
                        for _ in 0..cmd.simulations {
                            monte.simulate_once();
                        }

                        let best_move = monte.best_move();

                        let data: Vec<(String, u64)> = monte.move_scores().iter().map(|(mv, _, sims)| {
                            (chess_lib::fmt::format_move(*mv), *sims as u64)
                        }).collect();

                        let chart_data: Vec<(&str, u64)> = data.iter().map(|(k, v)| (k.as_str(), *v)).collect();

                        terminal.draw(|f| {
                            let chunks = Layout::default()
                                .direction(Direction::Vertical)
                                .margin(2)
                                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                                .split(f.size());

                            let top_chunks = Layout::default()
                                .direction(Direction::Horizontal)
                                .margin(0)
                                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                                .split(chunks[0]);

                            let barchart = BarChart::default()
                                .block(Block::default().title("Move Evaluations").borders(Borders::ALL))
                                .data(&chart_data)
                                .bar_width(5)
                                .bar_style(Style::default().fg(Color::Yellow))
                                .value_style(Style::default().fg(Color::Black).bg(Color::Yellow));

                            let chess_board = board::ChessBoard::with_highlight(draw_state.clone(), best_move);

                            f.render_widget(barchart, chunks[1]);
                            f.render_widget(chess_board, top_chunks[0]);
                        })?;

                        loop {
                            let k = stdin.next();
                            match k {
                                Some(Ok(Key::Char('q'))) => return Ok(()),
                                None => break,
                                _ => (),
                            }
                        }
                    }
                },
                _ => panic!("Invalid engine"),
            }
        },
        SubCommand::Magic(cmd) => {
            let (maskgen, movegen, target): (
                fn (chess_lib::types::BitCoord) -> chess_lib::types::BitBoard,
                fn (chess_lib::types::BitCoord, chess_lib::types::BitBoard) -> chess_lib::types::BitBoard,
                usize,
            ) = match cmd.piece.as_str() {
                "rook" => (chess_lib::magic::rook_mask, chess_lib::magic::rook_moves, 2056),
                "bishop" => (chess_lib::magic::bishop_mask, chess_lib::magic::bishop_moves, 128),
                _ => panic!("Unknown piece: {}", cmd.piece),
            };

            println!("=== {} ===", cmd.piece);
            let mut total_size = 0;
            for c in 0..64 {
                let mut best: u64 = 0;
                let mut best_size: usize = usize::MAX;

                let mut iterations = 0;
                let coord = chess_lib::types::BitCoord(1 << c);
                let mask = maskgen(coord);
                let moves = chess_lib::magic::generate_moves(coord, mask, movegen);

                while iterations < 10_000 && best_size > target {
                    iterations += 1;
                    let magic = rand::random::<u64>();
                    match chess_lib::magic::Magic::generate(magic, mask, &moves) {
                        Some(m) => {
                            let size = m.size();
                            if best_size == 0 || size < best_size {
                                best_size = size;
                                best = magic;
                            }
                        },
                        None => (),
                    }
                }

                println!("0x{:016x},  // {}[{}]", best, c, best_size);
                total_size += best_size;
            }

            println!("Total size: {} bytes", total_size * 8);

            Ok(())
        },
    }
}
