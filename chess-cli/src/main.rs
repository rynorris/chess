use std::io;
use std::time::Instant;
use clap::{AppSettings, Clap};

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
    fen: String,

    #[clap(short)]
    depth: u32,

    #[clap(short)]
    tt_bits: Option<u8>,
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
            let state = chess_lib::fen::load_fen(&cmd.fen);
            let mbb = chess_lib::magic::MagicBitBoards::default();
            let chess = chess_ai::chess::Chess::new(state, &mbb);
            let tt_size = 1 << cmd.tt_bits.unwrap_or(28);
            let mut alphabeta = chess_ai::minimax::AlphaBeta::new(chess_ai::eval::evaluate, tt_size);

            let before = Instant::now();
            let mut results = alphabeta.evaluate(&chess, cmd.depth);
            let after = Instant::now();

            results.sort_by(|(_, a), (_, b)| b.partial_cmp(a).unwrap());
            results.into_iter().for_each(|(m, s)| {
                println!("{}: {:.2}", chess_lib::fmt::format_move(m), (s as f64) / 100f64);
            });

            let duration = after - before;
            println!("Took: {}s", duration.as_secs_f32());

            println!("TT fill rate: {:.2}", alphabeta.tt_stats().fill_rate());
            println!("TT hit rate: {:.2}", alphabeta.tt_stats().hit_rate());
            println!("TT collision rate: {:.2}", alphabeta.tt_stats().collision_rate());

            Ok(())
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
