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
    #[clap(short, long)]
    fen: String,

    #[clap(short, long)]
    depth: u8,
}

#[derive(Clap)]
struct Analyze {
    #[clap(short, long)]
    fen: String,

    #[clap(short, long)]
    depth: u32,

    #[clap(short, long)]
    tt_bits: Option<u8>,
}

#[derive(Clap)]
struct Magic {
    #[clap(short, long)]
    piece: String,

    #[clap(short, long, default_value = "1_000")]
    iterations: u32,

    #[clap(short, long)]
    target: Option<usize>,

    #[clap(short, long)]
    continuous: bool,
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
            let (mv, score) = alphabeta.evaluate(&chess, cmd.depth);
            let after = Instant::now();

            println!("{}: {:.2}", chess_lib::fmt::format_move(mv), (score as f64) / 100f64);

            let duration = after - before;
            println!("Took: {}s", duration.as_secs_f32());

            println!("TT fill rate: {:.2}", alphabeta.tt_stats().fill_rate());
            println!("TT hit rate: {:.2}", alphabeta.tt_stats().hit_rate());
            println!("TT collision rate: {:.2}", alphabeta.tt_stats().collision_rate());

            Ok(())
        },
        SubCommand::Magic(cmd) => {
            let default_bbs = chess_lib::magic::MagicBitBoards::default();
            let (maskgen, movegen, target): (
                fn (chess_lib::types::BitCoord) -> chess_lib::types::BitBoard,
                fn (chess_lib::types::BitCoord, chess_lib::types::BitBoard) -> chess_lib::types::BitBoard,
                usize,
            ) = match cmd.piece.as_str() {
                "rook" => (chess_lib::magic::rook_mask, chess_lib::magic::rook_moves, 2056),
                "bishop" => (chess_lib::magic::bishop_mask, chess_lib::magic::bishop_moves, 128),
                _ => panic!("Unknown piece: {}", cmd.piece),
            };

            let mut bests: Vec<u64> = Vec::with_capacity(64);
            let mut best_sizes: Vec<usize> = Vec::with_capacity(64);
            for c in 0..64 {
                let coord = chess_lib::types::BitCoord(1 << c);
                let magic = match cmd.piece.as_str() {
                    "rook" => default_bbs.rook(coord),
                    "bishop" => default_bbs.bishop(coord),
                    _ => panic!("Unknown piece: {}", cmd.piece),
                };
                bests.push(magic.magic());
                best_sizes.push(magic.size());
            }

            println!("=== {} ({}, iterations={}, target={}) ===", cmd.piece, if cmd.continuous { "continuous" } else { "single" }, cmd.iterations, cmd.target.unwrap_or(target));
            loop {
                for c in 0..64 {
                    let mut iterations = 0;
                    let mut improved = false;
                    let coord = chess_lib::types::BitCoord(1 << c);
                    let mask = maskgen(coord);
                    let moves = chess_lib::magic::generate_moves(coord, mask, movegen);

                    let default = match cmd.piece.as_str() {
                        "rook" => default_bbs.rook(coord),
                        "bishop" => default_bbs.bishop(coord),
                        _ => panic!("Unknown piece: {}", cmd.piece),
                    };

                    while iterations < cmd.iterations && best_sizes[c] > cmd.target.unwrap_or(target) {
                        iterations += 1;
                        let magic = rand::random::<u64>();
                        match chess_lib::magic::Magic::generate(magic, mask, &moves) {
                            Some(m) => {
                                let size = m.size();
                                if size < best_sizes[c] {
                                    bests[c] = magic;
                                    best_sizes[c] = size;
                                    improved = true;
                                }
                            },
                            None => (),
                        }
                    }

                    if improved {
                        println!("0x{:016x},  // {}[{}] !! (was {})", bests[c], c, best_sizes[c], default.size());
                    } else if !cmd.continuous {
                        println!("0x{:016x},  // {}[{}]", bests[c], c, best_sizes[c]);
                    }
                }

                if cmd.continuous {
                    println!("Total size: {} bytes", best_sizes.iter().sum::<usize>() * 8);
                } else {
                    break;
                }
            }

            println!("Total size: {} bytes", best_sizes.iter().sum::<usize>() * 8);

            Ok(())
        },
    }
}
