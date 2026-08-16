use crate::chess::board::Board;
use crate::chess::move_generator::GEN_ALL;
use crate::chess::move_generator::MoveGenerator;
use crate::chess::move_list::MoveList;
use crate::chess::move_ply;
use crate::chess::move_ply::MovePly;
use crate::chess::types::color::Color;
use crate::engine::arbiter::Arbiter;
use crate::engine::perft::{BULK_PERFT, PERFT, TT_PERFT, perft};
use crate::engine::search::Searcher;
use crate::engine::search_limits::SearchLimits;
use crate::engine::transposition::Transposition;
use crate::engine::types::match_result::MatchResult;
use crate::uci::commands::{Commands, OptionsType};
use crate::uci::option_table::print_option_table;
use crate::uci::parser;
use std::env;
use std::fs::File;
use std::io::Read;
use std::process::exit;
use std::sync::Arc;

mod chess;
mod engine;
mod general;
pub mod precomputed;
mod uci;

const START_POS: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

const HELP_MSG: &str = "\nA fairly generic rust engine supporting the UCI protocol.\n\
Commands are the same as the uci protocol, except perft which can be called by perft <depth>\n\
Some UCI features are yet to be implemented.\n\
";

const AUTHOR: &str = "Harry Phillips";
const NAME: &str = "Generic Rust UCI Engine";

fn main() {
    // run_self_play();
    // dump_bins();
    // test_code();
    // return;
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        if args[1] == "pgo-test" {
            run_self_play();
        }

        if args[1] == "pgo-perft" {
            run_perft_pgo();
        }
        return;
    }

    println!("{NAME} by {AUTHOR}\n");

    let mut current_fen: String = START_POS.to_string();
    let mut board = Board::default();

    board.new(&current_fen);

    let mut tt_size = 64;
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

            Commands::Perft { depth } => {
                perft::<PERFT>(&mut board, depth as u8);
            }

            Commands::TTPerft { depth } => {
                perft::<TT_PERFT>(&mut board, depth as u8);
            }

            Commands::BulkPerft { depth } => {
                perft::<BULK_PERFT>(&mut board, depth as u8);
            }
            Commands::Position { fen, moves } => {
                current_fen = fen;
                board = Board::default();
                board.new(&current_fen);
                if let Some(str_moves) = moves {
                    for str_move in str_moves {
                        board.make_move::<true>(&move_ply::uci_move_parser(&str_move, &board))
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

                Searcher::search_start(&tt, &board, &search_limits, num_threads);
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
                    println!("Unknown command: '{line}'. Type help for more information.\n")
                }
            }

            _ => {}
        }
    }
}

/* fn test_code() {
    use cozy_chess;

    fn perft(
        my_board: &mut Board,
        cozy_board: cozy_chess::Board,
        depth: u8,
        mut num_nodes: u64,
    ) -> u64 {
        let mut cozy_move_list = Vec::new();
        cozy_board.generate_moves(|moves| {
            cozy_move_list.extend(moves);
            false
        });

        let mut move_list = MoveList::default();
        MoveGenerator::<GEN_ALL>::generate(my_board, &mut |mut piece_moves| {
            move_list.add_piece_moves(&mut piece_moves);
        });

        if cozy_move_list.len() != move_list.move_count() {
            println!("{cozy_board}");
            println!("{my_board}");

            for past_state in my_board.past_board_states().unwrap() {
                println!("Played: {}", past_state.played);
            }
            println!(
                "mine {} it {}",
                move_list.move_count(),
                cozy_move_list.len()
            );
            for curr_move in move_list.iter() {
                // if my_board.piece_at(curr_move.from()).is_pawn() {
                println!("{curr_move}");
                // }
            }

            panic!()
        }

        if depth == 1 {
            return move_list.move_count() as u64;
        }
        for curr_move in move_list.iter() {
            let mut cozy_board = cozy_board.clone();
            let mut move_as_str = curr_move.to_string();

            if my_board.piece_at(Square::E1).is_king() {
                match move_as_str.as_str() {
                    "e1g1" => move_as_str = "e1h1".to_string(),
                    "e1c1" => move_as_str = "e1a1".to_string(),
                    _ => {}
                }
            }

            if my_board.piece_at(Square::E8).is_king() {
                match move_as_str.as_str() {
                    "e8g8" => move_as_str = "e8h8".to_string(),
                    "e8c8" => move_as_str = "e8a8".to_string(),
                    _ => {}
                }
            }

            my_board.make_move(curr_move);
            cozy_board.play_unchecked(move_as_str.parse().unwrap());
            let search_nodes = perft(my_board, cozy_board.clone(), depth - 1, 0);

            num_nodes += search_nodes;
            my_board.undo_move();
        }

        num_nodes
    }

    let position = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";
    // let position = "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8";
    // let position = START_POS;
    let mut my_board = Board::default();
    my_board.new(position);
    let cozy_board = cozy_chess::Board::from_fen(position, false).unwrap();

    perft(&mut my_board, cozy_board, 5, 0);
} */

/* fn cozy_perft_bulk(depth: u8) {
    use cozy_chess;
    use std::time::Instant;

    fn perft_bulk(board: &cozy_chess::Board, depth: u8) -> u64 {
        let mut nodes = 0;
        match depth {
            0 => nodes += 1,
            1 => {
                board.generate_moves(|moves| {
                    nodes += moves.len() as u64;
                    false
                });
            }
            _ => {
                board.generate_moves(|moves| {
                    for mv in moves {
                        let mut board = board.clone();
                        board.play_unchecked(mv);
                        let child_nodes = perft_bulk(&board, depth - 1);
                        nodes += child_nodes;
                    }
                    false
                });
            }
        }
        nodes
    }
    let timer = Instant::now();
    let board = cozy_chess::Board::default();

    let num_nodes = perft_bulk(&board, depth);

    println!("nodes: {num_nodes}");
    println!("elapsed {}", (timer.elapsed().as_secs_f64()));
    println!("nps {}", num_nodes as f64 / (timer.elapsed().as_secs_f64()));
} */

// test code in case i need to check if something is broken
pub fn run_perft_pgo() {
    let mut board = Board::default();
    const PERFT_TYPE: u8 = BULK_PERFT;

    board.new("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    perft::<PERFT_TYPE>(&mut board, 7);

    board = Board::default();
    board.new("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - ");
    perft::<PERFT_TYPE>(&mut board, 6);

    board = Board::default();
    board.new("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1");
    perft::<PERFT_TYPE>(&mut board, 8);

    board = Board::default();
    board.new("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1");
    perft::<PERFT_TYPE>(&mut board, 6);

    board = Board::default();
    board.new("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8");
    perft::<PERFT_TYPE>(&mut board, 6);

    board = Board::default();
    board.new("r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10");
    perft::<PERFT_TYPE>(&mut board, 6);
}

pub fn run_self_play() {
    use std::sync::atomic::Ordering;

    let mut fen_file = File::open(
        "/home/harry-phillips/Desktop/antidraw_v2.1/AntiDraw_V2.1/UHO_2022/UHO_2022_+110_+119/UHO_2022_8mvs_+110_+119.epd",
    )
    .unwrap();
    let mut string = String::new();
    fen_file.read_to_string(&mut string).unwrap();
    let all_fens = string.split("\n").collect::<Vec<&str>>();

    for fen in all_fens.iter() {
        let tt = Arc::new(Transposition::new(16));

        let mut uci_moves_played: Vec<MovePly> = Vec::new();

        for i in 0.. {
            if i > 30 {
                return;
            }

            let mut board = Board::default();
            board.new(fen);

            for uci_move in &uci_moves_played {
                board.make_move::<true>(uci_move);
            }

            // println!("{board}");

            let mut valid_moves = MoveList::default();
            MoveGenerator::<GEN_ALL>::generate(&mut board, &mut |mut piece_moves| {
                valid_moves.add_piece_moves(&mut piece_moves);
            });

            let match_result = Arbiter::arbitrate(&mut board, &mut valid_moves);

            match match_result {
                MatchResult::Loss | MatchResult::Draw => break,
                MatchResult::NoResult => {}
            }

            let mut searcher = Searcher::new(&tt, &board, &SearchLimits::new(1000, 1000));

            let move_played = searcher.iterative_deepening();

            uci_moves_played.push(move_played);
            println!("{move_played}");

            tt.curr_depth.store(0, Ordering::Relaxed);
            tt.age();
        }
    }
}
