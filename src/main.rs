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

    let mut start_pos_moves = MoveList::default();
    MoveGenerator::generate(&board, &mut start_pos_moves);

    let timer = Instant::now();

    let mut all_nodes = 0;

    if depth == 1 {
        all_nodes = search(&mut board, depth, 0 )

    }else {

        for i in 0..start_pos_moves.move_count(){
            let cur_move = start_pos_moves.move_at(i);
            board.make_move(cur_move);
            let num_nodes = search(&mut board, depth-1, 0 );
            all_nodes += num_nodes;
            board.undo_move();
            println!("{cur_move}: {num_nodes}");
        }
    }


    let nodes_per_second = all_nodes as f64 / (timer.elapsed().as_secs_f64());
    let elapsed = timer.elapsed().as_secs_f64();

    println!("\nNodes searched: {all_nodes}");
    println!("Nodes per second: {nodes_per_second}");
    println!("Seconds elapsed: {elapsed}");
}