use std::time::Instant;
use clap::{AppSettings, Clap};

#[derive(Clap)]
#[clap(version = "0.1", author = "Ryan N. <rynorris@gmail.com>")]
#[clap(setting = AppSettings::ColoredHelp)]
struct Opts {
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Clap)]
enum SubCommand {
    BestMove(BestMove),
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
struct BestMove {
    #[clap(short)]
    fen: String,

    #[clap(short)]
    simulations: u64,
}

fn main() {
    let opts: Opts = Opts::parse();
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
        },
        SubCommand::BestMove(cmd) => {
            let state = chess_lib::fen::load_fen(&cmd.fen);

            let before = Instant::now();

            let mut monte = chess_ai::montecarlo::MCTS::new(state);

            for _ in 0..1000 {
                for _ in 0..cmd.simulations {
                    monte.simulate_once();
                }

                let best = monte.best_move();
                println!("The best move is: {}", chess_lib::fmt::format_move(best));

                let mut lines: Vec<String> = monte.move_scores().iter().map(|(mv, wins, sims)| {
                    format!("{}: {} / {}", chess_lib::fmt::format_move(*mv), wins, sims)
                }).collect();

                lines.sort();
                lines.iter().for_each(|l| println!("{}", l));
            }

            let after = Instant::now();
            let duration = after - before;
            println!("Took: {}s", duration.as_secs_f32());
        },
    }
}
