use crate::chess::board::Board;
use crate::chess::move_generator::MoveGenerator;
use crate::chess::move_list::MoveList;
use crate::chess::precomputed::data_dump::dump_bins;

mod chess;
mod general;

fn main() {
    let mut board = Board::default();
    board.new("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    
    let mut move_list = MoveList::default();
    MoveGenerator::generate(&board, &mut move_list);
    
    
    println!("Num moves in start pos: {}", move_list.move_count());
    
    for i in 0..move_list.move_count() {
        println!("{}", move_list.move_at(i).to_string());
    }
    
    
    println!("{}", board);
}
