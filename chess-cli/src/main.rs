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
    Divide(Divide),
}

#[derive(Clap)]
struct Divide {
    #[clap(short)]
    fen: String,

    #[clap(short)]
    depth: u8,
}

fn main() {
    let opts: Opts = Opts::parse();
    match opts.subcmd {
        SubCommand::Divide(div) => {
            let state = chess_lib::fen::load_fen(&div.fen);
            let counts = chess_lib::perft::divide(&state, div.depth);
            let mut lines: Vec<String> = counts.iter().map(|(k, v)| {
                return format!("{}: {}", k, v);
            }).collect();
            lines.sort();
            lines.iter().for_each(|l| println!("{}", l));
        },
    }
}
