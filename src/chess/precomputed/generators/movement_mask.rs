use crate::chess::consts;
use crate::chess::precomputed::generators::helpers::{create_dynamic_mask, create_static_mask, INCLUDE_EDGE};
use crate::chess::types::color::Color;
use crate::chess::types::square::Square;
use crate::chess::consts::NUM_SQUARES;

pub struct MovementMasks{
    pub knight: [u64; NUM_SQUARES],
    pub bishop: [u64; NUM_SQUARES],
    pub rook: [u64; NUM_SQUARES],
    pub queen: [u64; NUM_SQUARES],
    pub king: [u64; NUM_SQUARES],

    white_pawn_move: [u64; NUM_SQUARES],
    black_pawn_move: [u64; NUM_SQUARES],

    white_pawn_attacks: [u64; NUM_SQUARES],
    black_pawn_attacks: [u64; NUM_SQUARES],

    white_double_jump: [u64; NUM_SQUARES],
    black_double_jump: [u64; NUM_SQUARES],

}


impl MovementMasks {
    pub fn new() -> Self{

        let mut masks = MovementMasks{
            knight:  [0; NUM_SQUARES],
            bishop: [0; NUM_SQUARES],
            rook: [0; NUM_SQUARES],
            queen: [0; NUM_SQUARES],
            king: [0; NUM_SQUARES],

            white_pawn_move: [0; NUM_SQUARES],
            black_pawn_move: [0; NUM_SQUARES],

            white_pawn_attacks: [0; NUM_SQUARES],
            black_pawn_attacks: [0; NUM_SQUARES],

            white_double_jump: [0; NUM_SQUARES],
            black_double_jump: [0; NUM_SQUARES],

        };


        for i in 0..NUM_SQUARES {
            let square = Square::from(i as u8);

            let rank = square.rank();


            if rank.is_pawn_start(Color::White) { masks.white_double_jump[i] |=  1 << (i+16)}
            if rank.is_pawn_start(Color::Black) {  masks.black_double_jump[i] |=  1 << (i-16)}

            if !rank.is_pawn_promotion(Color::White) { masks.white_pawn_move[i] |=  1 << (i+8)}
            if !rank.is_pawn_promotion(Color::Black)  { masks.black_pawn_move[i] |=  1 << (i-8)}


            masks.white_pawn_attacks[i] |= create_static_mask(&consts::WHITE_PAWN_ATTACKS_DIRECTIONS, square);
            masks.black_pawn_attacks[i] |= create_static_mask(&consts::BLACK_PAWN_ATTACKS_DIRECTIONS, square);

            masks.knight[i] |= create_static_mask(&consts::KNIGHT_DIRECTIONS, square);
            masks.king[i] |= create_static_mask(&consts::KING_DIRECTIONS, square);

            masks.bishop[i] |= create_dynamic_mask::<INCLUDE_EDGE>(&consts::BISHOP_DIRECTIONS, square);
            masks.rook[i] |= create_dynamic_mask::<INCLUDE_EDGE>(&consts::ROOK_DIRECTIONS, square);
            masks.queen[i] |= create_dynamic_mask::<INCLUDE_EDGE>(&consts::QUEEN_DIRECTIONS, square);

        }


        masks
    }


    pub fn pawn_attacks(&self, color: Color, square: Square) -> u64{
        match color {
            Color::White => self.white_pawn_attacks[square as usize],
            Color::Black => self.black_pawn_attacks[square as usize]
        }
    }

    pub fn pawn_move(&self, color: Color, square: Square) -> u64{
        match color {
            Color::White => self.white_pawn_move[square as usize],
            Color::Black => self.black_pawn_move[square as usize]
        }
    }
    pub fn pawn_jump(&self, color: Color, square: Square) -> u64{
        match color {
            Color::White => self.white_double_jump[square as usize],
            Color::Black => self.black_double_jump[square as usize]
        }
    }





}