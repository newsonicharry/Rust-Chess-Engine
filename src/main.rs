use crate::chess::board::Board;
use crate::chess::precomputed::data_dump::dump_bins;

mod chess;
mod general;

fn main() {
    dump_bins();

    let mut board = Board::default();
    board.new("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    
    println!("{}", board);
}
