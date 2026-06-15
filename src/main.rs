use crate::chess::board::Board;
use crate::chess::move_generator::GEN_ALL;
use crate::chess::move_generator::MoveGenerator;
use crate::chess::move_list::MoveList;
use crate::chess::move_ply;
use crate::chess::move_ply::MovePly;
use crate::chess::types::color::Color;
use crate::engine::arbiter::Arbiter;
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
use std::time::Instant;

mod chess;
mod engine;
mod general;
pub mod precomputed;
mod uci;

const START_POS: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

const HELP_MSG: &str = "\nA fairly generic rust engine supporting the UCI protocol.\n\
Commands are the same as the uci protocol, except perft which can be called by perft <depth>\n\
Some UCI features are yet to be implemented.\
";

const AUTHOR: &str = "Harry Phillips";
const NAME: &str = "Generic Rust UCI Engine";

fn main() {
    // run_self_play();
    // return;

    // dump_bins();
    // println!("{}", std::mem::size_of::<TTEntry>());
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 && args[1] == "pgo-test" {
        run_self_play();
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
                perft(&mut board, depth as u8);
            }

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

struct PerftTT {
    entries: Box<[u128]>,
}

impl PerftTT {
    pub fn new(mb_size: usize) -> Self {
        let size_as_bytes = mb_size * 1024 * 1024;
        let num_entries = size_as_bytes / 16; // 16 bytes per entry 
        let entries: Box<[u128]> = vec![0; num_entries].into_boxed_slice();
        Self { entries }
    }

    fn zobrist_from_entry(entry: u128) -> u64 {
        (entry >> 64) as u64
    }

    fn num_nodes_from_entry(entry: u128) -> u64 {
        (entry as u64) >> 4
    }

    pub fn probe(&self, zobrist: u64, depth: u8) -> Option<u64> {
        let index = (zobrist & (self.entries.len() as u64 - 1)) as usize;
        let entry = self.entries[index];

        if Self::zobrist_from_entry(entry) == zobrist && (entry as u8 & 0b1111) == depth {
            return Some(Self::num_nodes_from_entry(entry));
        }

        None
    }

    pub fn update(&mut self, zobrist: u64, num_nodes: u64, depth: u8) {
        let index = (zobrist & (self.entries.len() as u64 - 1)) as usize;
        let entry = self.entries[index];

        if Self::zobrist_from_entry(entry) == 0 || depth >= (entry as u8 & 0b1111) {
            self.entries[index] = ((zobrist as u128) << 64)
                | ((num_nodes as u64) << 4) as u128
                | (depth & 0b1111) as u128;
        }
    }
}

fn perft(board: &mut Board, depth: u8) -> u64 {
    fn search(
        board: &mut Board,
        depth: u8,
        mut num_nodes: u64,
        transposition: &mut PerftTT,
        use_tt: bool,
    ) -> u64 {
        if let Some(tt_nodes) = transposition.probe(board.zobrist(), depth)
            && use_tt
        {
            return tt_nodes;
        }

        let mut move_list = MoveList::default();
        MoveGenerator::<GEN_ALL>::generate(board, &mut move_list);

        if depth == 1 {
            return move_list.move_count() as u64;
        }

        for cur_move in move_list.iter() {
            board.make_move(cur_move);
            let search_nodes = search(board, depth - 1, 0, transposition, use_tt);
            transposition.update(board.zobrist(), search_nodes, depth - 1);
            num_nodes += search_nodes;

            board.undo_move();
        }

        num_nodes
    }

    let mut transposition = PerftTT::new(128);

    let mut start_pos_moves = MoveList::default();
    MoveGenerator::<GEN_ALL>::generate(board, &mut start_pos_moves);

    let timer = Instant::now();

    let mut all_nodes = 0;

    if depth == 1 {
        all_nodes = search(board, depth, 0, &mut transposition, true)
    } else {
        for cur_move in start_pos_moves.iter() {
            board.make_move(cur_move);
            let num_nodes = search(board, depth - 1, 0, &mut transposition, true);
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

    all_nodes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_perft() {
        let mut board = Board::default();

        board.new("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
        assert_eq!(perft(&mut board, 7), 3_195_901_860);

        board = Board::default();
        board.new("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - ");
        assert_eq!(perft(&mut board, 5), 193_690_690);

        board = Board::default();
        board.new("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1");
        assert_eq!(perft(&mut board, 8), 3_009_794_393);

        board = Board::default();
        board.new("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1");
        assert_eq!(perft(&mut board, 6), 706_045_033);

        board = Board::default();
        board.new("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8");
        assert_eq!(perft(&mut board, 5), 89_941_194);

        board = Board::default();
        board.new("r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10");
        assert_eq!(perft(&mut board, 6), 6_923_051_137);
    }
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
                board.make_move(uci_move);
            }

            // println!("{board}");

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

            tt.age();
        }
    }
}
