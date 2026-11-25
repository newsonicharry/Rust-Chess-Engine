use std::mem;

use crate::chess::types::{color::Color, square::Square};

pub const NUM_FEATURES: usize = 768;
pub const HIDDEN_SIZE: usize = 1024;

// pub const L1: usize = 16;
// pub const L2: usize = 32;

pub const OUTPUT_BUCKETS: usize = 8;
pub const INPUT_BUCKETS: usize = 10;

#[rustfmt::skip]
pub const BUCKET_LAYOUT: [usize; 64] = [
    0, 1, 2, 3, 3, 2, 1, 0,
    4, 4, 5, 5, 5, 5, 4, 4,
    6, 6, 6, 6, 6, 6, 6, 6,
    7, 7, 7, 7, 7, 7, 7, 7,
    8, 8, 8, 8, 8, 8, 8, 8,
    8, 8, 8, 8, 8, 8, 8, 8,
    9, 9, 9, 9, 9, 9, 9, 9,
    9, 9, 9, 9, 9, 9, 9, 9
];

pub fn get_king_bucket(square: Square, color: Color) -> (usize, u8) {
    // vertically flip if black
    let square = match color {
        Color::White => square as usize,
        Color::Black => square.vert_flip() as usize,
    };

    // if the square is on the e-h files then set it to 7
    let flip = match square % 8 >= 4 {
        true => 7,
        false => 0,
    };

    // horizontally flip if on e-h files
    let new_square = square as u8 ^ flip;

    (BUCKET_LAYOUT[new_square as usize], flip)
}

pub const CR_MIN: i32 = 0;
pub const CR_MAX: i32 = 255;

pub const QA: i32 = 255;
pub const QAB: i32 = 255 * 64;

pub const EVAL_SCALE: i32 = 400;

#[repr(C)]
#[repr(align(64))]
pub struct Network {
    pub l0w: [[i16; NUM_FEATURES * HIDDEN_SIZE]; INPUT_BUCKETS],
    pub l0b: [i16; HIDDEN_SIZE],
    pub l1w: [[[i16; HIDDEN_SIZE]; 2]; OUTPUT_BUCKETS],
    pub l1b: [i16; OUTPUT_BUCKETS],
    // pub l0w: [[i16; NUM_FEATURES * HIDDEN_SIZE]; INPUT_BUCKETS],
    // pub l0b: [i16; HIDDEN_SIZE],

    // pub l1w: [[i8; HIDDEN_SIZE * L1]; OUTPUT_BUCKETS],
    // pub l1b: [[f32; L1]; OUTPUT_BUCKETS],

    // pub l2w: [[f32; L2 * L1]; OUTPUT_BUCKETS],
    // pub l2b: [[f32; L2]; OUTPUT_BUCKETS],

    // pub l3w: [[f32; L2]; OUTPUT_BUCKETS],
    // pub l3b: [f32; OUTPUT_BUCKETS],
}

pub static MODEL: Network =
    unsafe { mem::transmute(*include_bytes!("./bins/quantised1024_8_10.bin")) };
