use crate::chess::board::Board;
use crate::chess::move_generator::MoveGenerator;
use crate::chess::move_list::MoveList;
use std::time::Instant;

mod chess;
mod general;

const START_POS: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

fn main() {


    perft(START_POS, 7)
}


fn perft(fen: &str, depth: u8){
    fn search(board: &mut Board, depth: u8, mut num_nodes: u128) -> u128{
        let mut move_list = MoveList::default();
        MoveGenerator::generate(&board, &mut move_list);

        if depth == 1 {
            return move_list.move_count() as u128;
        }

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
    let nodes_per_second = num_nodes as f64 / (timer.elapsed().as_millis() as f64 / 1000f64).max(1f64);
    let elapsed = timer.elapsed().as_secs_f64();

    println!("Nodes: {num_nodes}");
    println!("NPS: {nodes_per_second}");
    println!("Elapsed: {elapsed}");


}