use crate::chess::bitboard::Bitboard;
use crate::chess::consts::{NUM_PIECES, NUM_SQUARES};
use crate::chess::piece_list::PieceList;
use crate::chess::types::color::Color;
use crate::chess::types::file::File;
use crate::chess::types::piece::{char_to_piece, Piece};
use crate::chess::types::rank::Rank;
use crate::chess::types::square::Square;
use std::fmt::Display;

pub struct Board{
    pub bitboards: [Bitboard; NUM_PIECES],
    pub piece_lists: [PieceList; NUM_PIECES],
    pub piece_squares: [Piece; NUM_SQUARES],

    pub side_to_move: Color,

    pub white_occupancy: u64,
    pub black_occupancy: u64,
    pub occupancy: u64,
}


impl Default for Board {
    fn default() -> Board {
        let board = Board{
            bitboards: [Bitboard::default(); NUM_PIECES],
            piece_lists: [PieceList::default(); NUM_PIECES],
            piece_squares: [Piece::NoPiece; NUM_SQUARES],

            side_to_move: Color::White,

            white_occupancy: 0,
            black_occupancy: 0,
            occupancy: 0,
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

                let cur_square = Square::from((cur_file, rank));
                cur_file.plus();
                let piece = char_to_piece(char).unwrap();

                self.add_piece(piece, cur_square);
            }
        }

    }

    fn add_piece(&mut self, piece: Piece, square: Square){
        self.bitboards[piece as usize].add_piece(square);
        self.piece_lists[piece as usize].add_piece(square);
        self.piece_squares[square as usize] = piece;
    }

    pub fn piece_at(&self, square: Square) -> Piece{
        self.piece_squares[square as usize]
    }

}

const TOP_SECTION: &str    = "┌─────┬─────┬─────┬─────┬─────┬─────┬─────┬─────┐";
const MIDDLE_SECTION: &str = "├─────┼─────┼─────┼─────┼─────┼─────┼─────┼─────┤";
const BOTTOM_SECTION: &str = "└─────┴─────┴─────┴─────┴─────┴─────┴─────┴─────┘";
const SIDE_BAR: &str = "│";


impl Display for Board {

    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {

        let mut pretty_print = TOP_SECTION.to_string();

        for i in 0..NUM_SQUARES {

            if i % 8 == 0 {

                if i != 0 {
                    pretty_print += &*(SIDE_BAR.to_owned() + "\n" + MIDDLE_SECTION + "\n");
                }
                else {
                    pretty_print += "\n";
                }

            }

            let square = Square::from((i ^ 56) as u8);
            let piece = self.piece_at(square);
            let piece_as_str = piece.to_string();

            pretty_print += &*(SIDE_BAR.to_owned() + "  " + piece_as_str.as_str() + "  ");

        }

        pretty_print += &*(SIDE_BAR.to_owned() + "\n" + BOTTOM_SECTION);

        write!(f, "{}", pretty_print)


    }
}