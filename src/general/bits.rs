use crate::chess::types::square::Square;
use crate::chess::bitboard::Bitboard;

#[inline(always)]
pub fn get_squares(bitboard: Bitboard) -> Vec<Square> {
    let mut all_squares = Vec::new();
    let mut mask = bitboard.0;
    
    while mask != 0 {
        let lsb = next(mask);
        all_squares.push( Square::from(lsb) );
        mask &= !(1 << lsb);
    }

    all_squares

}

#[inline(always)]
pub fn next(mask: u64) -> u8 {
    mask.trailing_zeros() as u8
}

#[inline(always)]
pub fn count(mask: u64) -> u8 {
    mask.count_ones() as u8
}

