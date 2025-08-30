use std::fmt::Display;
use crate::chess::bitboard::Bitboard;
use crate::chess::types::piece::{Piece, char_to_piece};
use crate::chess::consts::{NUM_PIECES, NUM_FILES, NUM_RANKS};
use crate::chess::types::file::File;
use crate::chess::types::rank::Rank;


pub struct Board{
    bitboards: [Bitboard; NUM_PIECES],
}


impl Default for Board {
    fn default() -> Board {
        let board = Board{
            bitboards: [Bitboard::default(); NUM_PIECES],
        };

        board
    }
}

impl Board{

    // fen string is considered accurate at this point
    // determining the fen strings accuracy is the job of the uci
    pub fn new(&mut self, fen: &str){

        let split_fen: Vec<&str> =  fen.split_whitespace().collect();

        let fen_sequence = split_fen.get(0).unwrap();
        let ranks= fen_sequence.split('/').collect::<Vec<&str>>();

        for (i, section) in ranks.iter().rev().enumerate() {
            let rank = Rank::from(i as u8);
            
            let mut cur_file = File::A;
            for char in section.chars() {
                if char.is_numeric() {
                    let num_skipped_files = char.to_digit(10).unwrap() as u8;
                    cur_file.add(num_skipped_files);
                    continue;
                }
                
                
                let piece = char_to_piece(char).unwrap();


            }
        }

    }


}


// impl Display for Board {
//     fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
//
//
//
//     }
// }