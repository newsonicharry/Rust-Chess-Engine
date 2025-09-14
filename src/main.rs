#![feature(integer_atomics)]

use crate::chess::board::Board;
use crate::chess::move_generator::MoveGenerator;
use crate::chess::move_generator::GEN_ALL;
use crate::chess::move_list::MoveList;
use crate::chess::move_ply;
use crate::chess::types::color::Color;
use crate::engine::search::search_start;
use crate::engine::search_limits::SearchLimits;
use crate::engine::transposition::Transposition;
use crate::uci::commands::{Commands, OptionsType};
use crate::uci::option_table::print_option_table;
use crate::uci::parser;
use std::io::Read;
use std::process::exit;
use std::sync::Arc;
use std::time::Instant;

mod chess;
mod general;
mod engine;
mod uci;


const START_POS: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

const HELP_MSG: &str = "\nA fairly generic rust engine supporting the UCI protocol.\n\
Commands are the same as the uci protocol, except perft which can be called by perft <depth>\n\
Some UCI features are yet to be implemented.\
";

const AUTHOR: &str = "Harry Phillips";
const NAME: &str = "Generic Rust UCI Engine";

fn main() {
    println!("{NAME} by {AUTHOR}\n");

    let mut current_fen: String = START_POS.to_string();
    let mut board = Board::default();
    board.new(&current_fen);

    let mut tt_size = 16;
    let mut tt = Arc::new(Transposition::new(tt_size));

    let mut num_threads = 1;

    loop {
        let mut input: String = String::new();
        std::io::stdin().read_line(&mut input).expect("Failed to read input line.");

        let uci_command = parser::UCIParser::parse(&input);
        match uci_command {
            Commands::Uci => {
                println!("id name {NAME}\nid author {AUTHOR}\n");
                print_option_table();
                println!("uciok");

            },
            Commands::IsReady => println!("readyok"),
            Commands::Quit => exit(1),

            Commands::Help => println!("{}", HELP_MSG),

            Commands::UciNewGame => {
                tt = Arc::new(Transposition::new(tt_size));
                current_fen = START_POS.to_string();
                board.new(&current_fen);
            },

            Commands::Perft { depth } => { perft(&mut board, depth as u8) }

            Commands::Position {fen, moves } => {
                current_fen = fen;
                board = Board::default();
                board.new(&current_fen);
                if let Some(str_moves) = moves{
                    for str_move in str_moves{
                        board.make_move(&move_ply::uci_move_parser(str_move, &board))
                    }
                }
            }

            Commands::Go { move_time, wtime, btime, winc, binc, moves_to_go} => {

                let mut hard_think_time: u32 = 1000;
                let soft_think_time;


                let mut moves_left: u32 = 20;

                if let Some(moves_to_go) = moves_to_go { moves_left = moves_to_go; }

                match board.side_to_move() {
                    Color::White => {
                        if let Some(wtime) = wtime { hard_think_time = wtime / moves_left; }
                        if let Some(winc) = winc { hard_think_time += winc; }
                    }
                    Color::Black => {
                        if let Some(btime) = btime { hard_think_time = btime / moves_left; }
                        if let Some(binc) = binc { hard_think_time += binc; }
                    }
                }

                if let Some(move_time) = move_time {
                    hard_think_time = move_time;
                    soft_think_time = move_time;
                }else {
                    soft_think_time = (hard_think_time as f64 * 0.6f64) as u32;
                }

                let search_limits = SearchLimits::new(hard_think_time, soft_think_time);
                search_start(num_threads, board, &tt, &search_limits);
            }

            Commands::SetOption {options_type} => {
                match options_type {
                    OptionsType::Spin { name, value } => {
                        match name.as_str() {
                            "Threads" => { num_threads = value as usize }
                            "Hash" => { tt_size = value;  tt = Arc::new(Transposition::new(tt_size)); }
                            _ => unreachable!()
                        }

                    }

                    OptionsType::Button { name } => {
                        match name.as_str() {
                            "Clear Hash" => { tt = Arc::new(Transposition::new(tt_size)); }
                            _ => unreachable!()
                        }
                    }

                }

            }

            Commands::Unknown(line) =>  {
                if line != "\r\n" {
                    println!("Unknown command: '{line}'. Type help for more information.")
                }
            },

            _ => {}
        }



    }


}




fn perft(board: &mut Board, depth: u8){
    fn search(board: &mut Board, depth: u8, mut num_nodes: u128) -> u128{
        // if depth == 0 {
        //     return 1;
        // }

        let mut move_list = MoveList::default();
        MoveGenerator::<GEN_ALL>::generate(board, &mut move_list);

        if depth == 1 {
            return move_list.move_count() as u128;
        }

        for cur_move in move_list.iter(){
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
        all_nodes = search(board, depth, 0 )

    }else {
        for cur_move in start_pos_moves.iter(){
            board.make_move(cur_move);
            let num_nodes = search(board, depth-1, 0 );
            all_nodes += num_nodes;
            board.undo_move();
            println!("{cur_move}: {num_nodes}");
        }
    }


    let nodes_per_second = all_nodes as f64 / (timer.elapsed().as_secs_f64());
    let elapsed = timer.elapsed().as_secs_f64();

    println!("\nNodes searched: {all_nodes}");
    println!("Nodes per second: {nodes_per_second}");
    println!("Seconds elapsed: {elapsed}\n");
}

// test code in case i need to check if something is broken
// pub fn run_self_play(){
//
//     let mut fen_file = File::open("C:/Users/Harry/Desktop/UHO_2022_8mvs_+110_+119.epd").unwrap();
//     let mut string = String::new();
//     fen_file.read_to_string(&mut string).unwrap();
//     let all_fens = string.split("\n").collect::<Vec<&str>>();
//
//     for fen in all_fens {
//         let mut tt = Arc::new(Transposition::new(16));
//
//         let mut uci_moves_played: Vec<MovePly> = Vec::new();
//         println!("fen: {fen}");
//         loop{
//             let mut board = Board::default();
//             board.new(fen);
//
//             for uci_move in &uci_moves_played {
//                 board.make_move(uci_move);
//             }
//
//             let mut nnue = NNUE::default();
//             nnue.new(&mut board);
//
//             let mut valid_moves = MoveList::default();
//             MoveGenerator::<GEN_ALL>::generate(&mut board, &mut valid_moves);
//
//             let match_result = Arbiter::arbitrate(&mut board, &mut valid_moves);
//             match match_result {
//                 MatchResult::Loss | MatchResult::Draw => {break}
//                 MatchResult::NoResult => {}
//             }
//
//             let move_played = iterative_deepening(&mut board, &tt, &mut nnue, &SearchLimits::new(100, 100));
//             uci_moves_played.push(move_played);
//             println!("{move_played}");
//
//         }
//
//
//     }
//     println!("{}", string);
//
//
// }