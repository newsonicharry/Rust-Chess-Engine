use crate::chess::board::Board;
use crate::chess::types::file::File;

mod chess;
mod general;

fn main() {


    let mut board = Board::default();
    board.new("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    
    println!("{}", board);
}
