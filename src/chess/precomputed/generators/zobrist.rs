use rand::Rng;
use crate::chess::board::Board;
use crate::chess::consts::{NUM_SQUARES, NUM_FILES, NUM_PIECES};
use crate::chess::types::color::Color;
use crate::chess::types::color::Color::{White, Black};
use crate::chess::types::file::File;
use crate::chess::types::piece::Piece;
use crate::chess::types::square::Square;
use crate::general::bits;


const WHITE_SHORT_CASTLE: usize = 0;
const WHITE_LONG_CASTLE: usize = 1;
const BLACK_SHORT_CASTLE: usize = 2;
const BLACK_LONG_CASTLE: usize = 3;
const NUM_CASTLE: usize = 4;
pub struct Zobrist{
    squares:  [[u64; NUM_SQUARES]; NUM_PIECES],
    double_jump: [u64; NUM_FILES],
    castling_rights: [u64; NUM_CASTLE],
    side_to_move: u64,
}

impl Zobrist{
    pub fn new() -> Self{
        let mut zobrist = Zobrist{
            squares: [[0; NUM_SQUARES]; NUM_PIECES],
            double_jump: [0; NUM_FILES],
            castling_rights: [0; NUM_CASTLE],
            side_to_move: 0,
        };

        for piece_index in 0..NUM_PIECES{
            for square_index in 0..NUM_SQUARES  { 
                zobrist.squares[piece_index][square_index] = Self::gen_rand();
            }
        }
        for i in 0..NUM_FILES { zobrist.double_jump[i] = Self::gen_rand(); }
        for i in 0..4    { zobrist.castling_rights[i] = Self::gen_rand(); }

        zobrist.side_to_move = Self::gen_rand();

        zobrist


    }
    
    pub fn hash_from_board(&self, board: &Board) -> u64{
        let mut final_zobrist: u64 = 0;

        let squares_with_pieces = bits::all_squares(board.all_occupancy());
        for square in squares_with_pieces {
            let piece = board.piece_at(square);
            final_zobrist ^= self.squares[piece as usize][square as usize];
        }
        
        if board.has_short_castle_rights(White) { final_zobrist ^= self.castling_rights[WHITE_SHORT_CASTLE]; }
        if board.has_long_castle_rights(White) { final_zobrist ^= self.castling_rights[WHITE_LONG_CASTLE]; }
        if board.has_short_castle_rights(Black) { final_zobrist ^= self.castling_rights[BLACK_SHORT_CASTLE]; }
        if board.has_long_castle_rights(Black) { final_zobrist ^= self.castling_rights[BLACK_LONG_CASTLE]; }

        if let Some(en_passant_file) = board.en_passant_file() { 
            final_zobrist ^= self.double_jump[en_passant_file as usize];
        }
        
        final_zobrist ^= self.side_to_move;
        
        final_zobrist
        
    }

    fn gen_rand() -> u64{
        rand::thread_rng().r#gen::<u64>()
    }


    pub fn square_zobrist(&self, piece: Piece, square: Square) -> u64{
        self.squares[piece as usize][square as usize]
    }

    pub fn short_castle(&self, color: Color) -> u64{
        match color {
            White => self.castling_rights[WHITE_LONG_CASTLE],
            Black => self.castling_rights[BLACK_LONG_CASTLE],
        }
    }
    
    pub fn long_castle(&self, color: Color) -> u64{
        match color { 
            White => self.castling_rights[WHITE_SHORT_CASTLE],
            Black => self.castling_rights[BLACK_SHORT_CASTLE],
        }
    }
    
    pub fn pawn_jump(&self, file: File) -> u64{
        self.double_jump[file as usize]
    }
    
    pub fn side_to_move(&self) -> u64{
        self.side_to_move
    }
}