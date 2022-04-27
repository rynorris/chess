use std::io;
use std::time::Instant;

use clap::{AppSettings, Clap};
use crossbeam::channel;
use threadpool::ThreadPool;

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

    #[clap(short, long)]
    iterations: Option<u32>,

    #[clap(short, long, default_value = "1")]
    workers: usize,
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
            let (maskgen, movegen): (
                fn (chess_lib::types::BitCoord) -> chess_lib::types::BitBoard,
                fn (chess_lib::types::BitCoord, chess_lib::types::BitBoard) -> chess_lib::types::BitBoard,
            ) = match cmd.piece.as_str() {
                "rook" => (chess_lib::magic::rook_mask, chess_lib::magic::rook_moves),
                "bishop" => (chess_lib::magic::bishop_mask, chess_lib::magic::bishop_moves),
                _ => panic!("Unknown piece: {}", cmd.piece),
            };

            let mut bests: Vec<chess_lib::magic::Magic> = Vec::with_capacity(64);
            for c in 0..64 {
                let coord = chess_lib::types::BitCoord(1 << c);
                let magic = match cmd.piece.as_str() {
                    "rook" => default_bbs.rook(coord),
                    "bishop" => default_bbs.bishop(coord),
                    _ => panic!("Unknown piece: {}", cmd.piece),
                };
                bests.push(magic.clone());
            }

            let (result_tx, result_rx) = channel::unbounded::<(usize, chess_lib::magic::Magic)>();

            let pool = ThreadPool::new(cmd.workers);

            println!("=== {} (iterations={}, workers={}) ===", cmd.piece, cmd.iterations.unwrap_or(0), cmd.workers);
            let mut iteration = 0;

            loop {
                iteration += 1;
                let iteration_start = Instant::now();
                for c in 0..64 {
                    let best = bests[c].clone();
                    let tx = result_tx.clone();
                    pool.execute(move|| {
                        let coord = chess_lib::types::BitCoord(1 << c);
                        let mask = maskgen(coord);
                        let moves = chess_lib::magic::generate_moves(coord, mask, movegen);

                        for _ in 0..100 {
                            let magic = rand::random::<u64>();
                            match chess_lib::magic::Magic::generate(magic, mask, &moves) {
                                Some(m) => {
                                    let size = m.size();
                                    if size < best.size() {
                                        tx.send((c, m)).expect("able to report results");
                                        return;
                                    }
                                },
                                None => (),
                            }
                        }
                        tx.send((c, best)).expect("able to report results");
                    });
                }

                result_rx.iter().take(64).for_each(|(c, m)| {
                    if m.size() < bests[c].size() {
                        println!("0x{:016x},  // {}[{}] !! (was {})", m.magic(), c, m.size(), bests[c].size());
                        bests[c] = m;
                    }
                });

                let duration = iteration_start.elapsed();
                let per_second = (64 * 100 * 1000) / duration.as_millis();

                println!("[#{}, took {:.1}s, {} magics/s] Total size: {} bytes", iteration, (duration.as_millis() as f64) / 1000.0, per_second, bests.iter().map(|m| m.size()).sum::<usize>() * 8);

                if iteration == cmd.iterations.unwrap_or(0) {
                    break;
                }
            }

            Ok(())
        },
    }
}
