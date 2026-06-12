use crate::chess::board::Board;
use crate::chess::move_generator::GEN_ALL;
use crate::chess::move_generator::MoveGenerator;
use crate::chess::move_list::MoveList;
use crate::chess::move_ply;
use crate::chess::move_ply::MovePly;
use crate::chess::move_ply::uci_move_parser;
use crate::chess::types::color::Color;
use crate::engine::arbiter::Arbiter;
use crate::engine::eval::nnue::NNUE;
use crate::engine::search::Searcher;
use crate::engine::search_limits::SearchLimits;
use crate::engine::transposition::TTEntry;
use crate::engine::transposition::Transposition;
use crate::engine::types::match_result::MatchResult;
use crate::precomputed::data_dump::dump_bins;
use crate::uci::commands::{Commands, OptionsType};
use crate::uci::option_table::print_option_table;
use crate::uci::parser;
use std::fs::File;
use std::io::Read;
use std::process::exit;
use std::sync::Arc;
use std::time::Instant;

mod chess;
mod engine;
mod general;
pub mod precomputed;
mod uci;

/*
a8a7 d8d7 a7a8 d7d8 a8a7 c8d7 a7a8 d7c8 a6a7 d6d7 a7a6 d4d3 h2g3 b3b5 g7a7 e6d6 c7g7 f7e6 c8c7 g8f7 c7c8 h7g8 c1c7 f3b3 g3d6 f8f3 d3f3 f4f3 h4g3 f5f4 g3h4 h5h4 d1c1 d5f5 f2g3 e8f8 d2d1 d7d6 e1f2 a8e8 b1d3 h8h7 d3d2 h7h5 a2b1 f8a8 f2e1 g8h8 d2a2 b5d7 e1f2 d7d5 d1d2 b7d7 d2e1 b8f8 e3f3 d5b5 e1e3 f7b7 g1h2 a7a5 c1d2 b7d5 c4d5 d6d5 b2b3 b6b7 a3d3 c5d4 f3d4 f8f7 d5f7 f5d4 a1a3 d8b6 a6b7 a8b8 a5a6 e6f7 e4d5 c7d5 a4a5 c8e6 c3d5 e6d5 f1e1 h6f5 d3e4 f5e4 h2h3 e7e6 b1c3 e8g8 c2c4 f7f5 e1g1 g8h6 f1d3 a6c7 g1f3 b8a6 a2a4 d7d6 d4d5 c7c5 f2f4 f8g7 e2e4 g7g6 d2d4

d7e8 h5g4 e8d7 g4h5 d7e8 h5g4 e8d7 g4h5 c6d8 d4d5 d8e8 f1f2 g3f2 f3g4 h4g3 c1g5 h5h4 g2f3 g4f3 h2h3 c8g4 e3e4 e8g8 b2c3 b4c3 e1g1 c5b4 d2d4 g8e7 e2e3 d7d6 b1c3 f8c5 g1f3 h7h5 f1g2 b8c6 g2g3 e7e5 c2c4

*/

const START_POS: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

const HELP_MSG: &str = "\nA fairly generic rust engine supporting the UCI protocol.\n\
Commands are the same as the uci protocol, except perft which can be called by perft <depth>\n\
Some UCI features are yet to be implemented.\
";

const AUTHOR: &str = "Harry Phillips";
const NAME: &str = "Generic Rust UCI Engine";

fn main() {
    // let moves = "d7e8 h5g4 e8d7 g4h5 d7e8 h5g4 e8d7 g4h5 c6d8 d4d5 d8e8 f1f2 g3f2 f3g4 h4g3 c1g5 h5h4 g2f3 g4f3 h2h3 c8g4 e3e4 e8g8 b2c3 b4c3 e1g1 c5b4 d2d4 g8e7 e2e3 d7d6 b1c3 f8c5 g1f3 h7h5 f1g2 b8c6 g2g3 e7e5 c2c4";

    // let mut board = Board::default();
    // board.new(START_POS);

    // let mut tt = Arc::new(Transposition::new(16));
    // // let search_limits = SearchLimits::new(1000, 1000);

    // for (index, str_move) in moves.split_whitespace().rev().enumerate() {
    //     let mv = uci_move_parser(str_move, &board);
    //     println!("{}", board.zobrist());

    //     println!("{board}");

    //     println!("{index}");

    //     if index >= 36 {
    //         let search_limits = if index == 38 {
    //             SearchLimits::new(1000, 1000)
    //         } else {
    //             SearchLimits::new(1000, 1000)
    //         };
    //         let mut searcher = Searcher::new(&tt, &board, &search_limits);
    //         searcher.search_start(0);

    //         // tt = Arc::new(Transposition::new(16));
    //     }

    //     board.make_move(&mv);
    // }

    // let mut searcher = Searcher::new(&tt, &board, &SearchLimits::new(1000, 1000));
    // searcher.search_start(0);

    // board.undo_move();
    // board.undo_move();

    // println!("{}", board.zobrist());

    // let mut move_list = MoveList::default();
    // MoveGenerator::<GEN_ALL>::generate(&mut board, &mut move_list);
    // let position_result = Arbiter::arbitrate(&board, &move_list);

    // println!("{:?}", position_result);

    // println!("{board}");

    // return;
    // run_self_play();
    // dump_bins();
    // println!("{}", std::mem::size_of::<TTEntry>());
    println!("{NAME} by {AUTHOR}\n");

    let mut current_fen: String = START_POS.to_string();
    let mut board = Board::default();
    board.new(&current_fen);
    let mut tt_size = 16;
    let mut tt = Arc::new(Transposition::new(tt_size));

    let mut num_threads = 1;

    loop {
        let mut input: String = String::new();
        std::io::stdin()
            .read_line(&mut input)
            .expect("Failed to read input line.");

        let uci_command = parser::UCIParser::parse(&input);
        match uci_command {
            Commands::Uci => {
                println!("id name {NAME}\nid author {AUTHOR}\n");
                print_option_table();
                println!("uciok");
            }
            Commands::IsReady => println!("readyok"),
            Commands::Quit => exit(1),

            Commands::Help => println!("{}", HELP_MSG),

            Commands::UciNewGame => {
                tt = Arc::new(Transposition::new(tt_size));
                current_fen = START_POS.to_string();
                board.new(&current_fen);
            }

            Commands::Perft { depth } => perft(&mut board, depth as u8),

            Commands::Position { fen, moves } => {
                current_fen = fen;
                board = Board::default();
                board.new(&current_fen);
                if let Some(str_moves) = moves {
                    for str_move in str_moves {
                        board.make_move(&move_ply::uci_move_parser(&str_move, &board))
                    }
                }
            }

            Commands::Go {
                move_time,
                wtime,
                btime,
                winc,
                binc,
                moves_to_go,
            } => {
                let mut hard_think_time: u32 = 1000;
                let soft_think_time;

                let mut moves_left: u32 = 20;

                if let Some(moves_to_go) = moves_to_go {
                    moves_left = moves_to_go;
                }

                match board.side_to_move() {
                    Color::White => {
                        if let Some(wtime) = wtime {
                            hard_think_time = wtime / moves_left;
                        }
                        if let Some(winc) = winc {
                            hard_think_time += winc;
                        }
                    }
                    Color::Black => {
                        if let Some(btime) = btime {
                            hard_think_time = btime / moves_left;
                        }
                        if let Some(binc) = binc {
                            hard_think_time += binc;
                        }
                    }
                }

                if let Some(move_time) = move_time {
                    hard_think_time = move_time.saturating_sub(20).max(5);
                    soft_think_time = move_time.saturating_sub(20).max(5);
                } else {
                    soft_think_time = (hard_think_time as f64 * 0.6f64) as u32;
                }

                let search_limits = SearchLimits::new(hard_think_time, soft_think_time);

                let mut searcher = Searcher::new(&tt, &board, &search_limits);
                searcher.search_start(num_threads);
                // Searcher::search_start(num_threads, board, &tt, &search_limits);
            }

            Commands::SetOption { options_type } => match options_type {
                OptionsType::Spin { name, value } => match name.as_str() {
                    "Threads" => num_threads = value as usize,
                    "Hash" => {
                        tt_size = value;
                        tt = Arc::new(Transposition::new(tt_size));
                    }
                    _ => unreachable!(),
                },

                OptionsType::Button { name } => match name.as_str() {
                    "Clear Hash" => {
                        tt = Arc::new(Transposition::new(tt_size));
                    }
                    _ => unreachable!(),
                },
            },

            Commands::Unknown(line) => {
                if line != "\r\n" {
                    println!("Unknown command: '{line}'. Type help for more information.")
                }
            }

            _ => {}
        }
    }
}

fn perft(board: &mut Board, depth: u8) {
    fn search(board: &mut Board, depth: u8, mut num_nodes: u128) -> u128 {
        // if depth == 0 {
        //     return 1;
        // }

        let mut move_list = MoveList::default();
        MoveGenerator::<GEN_ALL>::generate(board, &mut move_list);

        if depth == 1 {
            return move_list.move_count() as u128;
        }

        for cur_move in move_list.iter() {
            board.make_move(cur_move);
            num_nodes += search(board, depth - 1, 0);
            board.undo_move();
        }

        num_nodes
    }

    let mut start_pos_moves = MoveList::default();
    MoveGenerator::<GEN_ALL>::generate(board, &mut start_pos_moves);

    let timer = Instant::now();

    let mut all_nodes = 0;

    if depth == 1 {
        all_nodes = search(board, depth, 0)
    } else {
        for cur_move in start_pos_moves.iter() {
            board.make_move(cur_move);
            let num_nodes = search(board, depth - 1, 0);
            all_nodes += num_nodes;
            board.undo_move();
            println!("{cur_move}: {num_nodes}");
        }
    }

    let nodes_per_second = all_nodes as f64 / (timer.elapsed().as_secs_f64());
    let elapsed = timer.elapsed().as_secs_f64();

    println!("\nNodes searched: {all_nodes}");
    println!("Nodes per second: {nodes_per_second:.0}");
    println!("Seconds elapsed: {elapsed:.3}\n");
}

// test code in case i need to check if something is broken
pub fn run_self_play() {
    let mut fen_file = File::open(
        "/home/harry-phillips/Desktop/antidraw_v2.1/AntiDraw_V2.1/UHO_2022/UHO_2022_+110_+119/UHO_2022_8mvs_+110_+119.epd",
    )
    .unwrap();
    let mut string = String::new();
    fen_file.read_to_string(&mut string).unwrap();
    let all_fens = string.split("\n").collect::<Vec<&str>>();

    for fen in all_fens {
        let tt = Arc::new(Transposition::new(16));

        let mut uci_moves_played: Vec<MovePly> = Vec::new();
        println!("fen: {fen}");
        loop {
            let mut board = Board::default();
            board.new(fen);

            for uci_move in &uci_moves_played {
                board.make_move(uci_move);
            }

            let mut valid_moves = MoveList::default();
            MoveGenerator::<GEN_ALL>::generate(&mut board, &mut valid_moves);

            let match_result = Arbiter::arbitrate(&mut board, &mut valid_moves);
            match match_result {
                MatchResult::Loss | MatchResult::Draw => break,
                MatchResult::NoResult => {}
            }

            let mut searcher = Searcher::new(&tt, &board, &SearchLimits::new(100, 100));
            let move_played = searcher.iterative_deepening();

            uci_moves_played.push(move_played);
            println!("{move_played}");
        }
    }
    println!("{}", string);
}
