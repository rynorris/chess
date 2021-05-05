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
    fen: String,

    #[clap(short)]
    simulations: u64,
}

fn main() -> Result<(), io::Error> {
    let opts: Opts = Opts::parse();
    let stdout = io::stdout().into_raw_mode()?;
    let mut stdin = termion::async_stdin().keys();
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    match opts.subcmd {
        SubCommand::Divide(div) => {
            let state = chess_lib::fen::load_fen(&div.fen);

            let before = Instant::now();
            let counts = chess_lib::perft::divide(&state, div.depth);
            let after = Instant::now();

            let mut lines: Vec<String> = counts.iter().map(|(k, v)| {
                return format!("{}: {}", k, v);
            }).collect();
            lines.sort();
            lines.iter().for_each(|l| println!("{}", l));

            let duration = after - before;
            println!("Took: {}s", duration.as_secs_f32());
            Ok(())
        },
        SubCommand::Analyze(cmd) => {
            terminal.clear()?;
            let state = chess_lib::fen::load_fen(&cmd.fen);
            let position_board = state.board.clone();

            let mut monte = chess_ai::montecarlo::MCTS::new(state);

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
                        .bar_width(9)
                        .bar_style(Style::default().fg(Color::Yellow))
                        .value_style(Style::default().fg(Color::Black).bg(Color::Yellow));

                    let chess_board = board::ChessBoard::with_highlight(position_board.clone(), best_move);

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
    }
}
