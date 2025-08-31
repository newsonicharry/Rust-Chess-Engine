use std::time::Instant;
use crate::chess::board::Board;
use crate::chess::move_generator::MoveGenerator;
use crate::chess::move_list::MoveList;
use crate::chess::precomputed::data_dump::dump_bins;

mod chess;
mod general;

const START_POS: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

fn main() {
    perft(START_POS, 3)
}


fn perft(fen: &str, depth: u8){
    fn search(board: &mut Board, depth: u8, mut num_nodes: u128) -> u128{

        if depth == 0 {
            return 1;
        }

        let mut move_list = MoveList::default();
        MoveGenerator::generate(&board, &mut move_list);

        for i in 0..move_list.move_count() {
            let cur_move = move_list.move_at(i);

            board.make_move(cur_move);
            num_nodes += search(board, depth - 1, 0);
            board.undo_move();
        }

        num_nodes

    }

    let mut board = Board::default();
    board.new(fen);

    let timer = Instant::now();

    let num_nodes = search(&mut board, depth, 0);

    let nodes_per_second = num_nodes / (timer.elapsed().as_millis() / 1000).max(1);
    let elapsed = timer.elapsed().as_secs();

    println!("Nodes: {num_nodes}");
    println!("NPS: {nodes_per_second}");
    println!("Elapsed: {elapsed}");


}