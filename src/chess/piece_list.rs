use crate::chess::bitboard::Bitboard;
use crate::chess::types::square::Square;
use crate::general::bits::all_squares;

const MAX_PIECE_LEN: usize = 10;

#[derive(Copy, Clone)]
pub struct PieceList {
    piece_indexes: [Square; MAX_PIECE_LEN],
    piece_count: u8,
    map: [u8; 64],
}


impl Default for PieceList {
    fn default() -> Self {
        let material_list = PieceList {
            piece_indexes: [Square::default(); MAX_PIECE_LEN],
            piece_count: 0,
            map: [0; 64]
        };

        material_list
    }
}

impl PieceList {

    pub fn new(&mut self, bitboard: Bitboard){
        let squares = all_squares(bitboard.0);

        for (i, square) in squares.iter().enumerate(){
            self.map[*square as usize] = i as u8;
            self.piece_indexes[i] = *square;
        }
        
        self.piece_count = squares.len() as u8;
    }

    pub fn add_piece(&mut self, square: Square){
        self.piece_indexes[self.piece_count as usize] = square;
        self.map[square as usize] = self.piece_count;
        self.piece_count += 1;
    }

    pub fn remove_piece(&mut self, square: Square){
        let piece_index = self.map[square as usize];

        self.piece_indexes[piece_index as usize] = self.piece_indexes[(self.piece_count-1) as usize];
        self.map[self.piece_indexes[piece_index  as usize] as usize] = piece_index;
        self.piece_count -= 1;
    }

    pub fn move_piece(&mut self, from_square: Square, to_square: Square){
        let piece_index = self.map[from_square as usize];
        self.piece_indexes[piece_index as usize] = to_square;
        self.map[to_square as usize] = piece_index;

    }

    pub fn count(&self) -> u8 {
        self.piece_count
    }

    pub fn indexes(&self) -> &[Square] {
        &self.piece_indexes[..(self.piece_count as usize)]
    }


}