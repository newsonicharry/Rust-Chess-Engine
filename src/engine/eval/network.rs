use std::mem;

pub const NUM_FEATURES: usize = 768;
pub const HIDDEN_SIZE: usize = 256;

pub const CR_MIN: i16 = 0;
pub const CR_MAX: i16 = 255;

pub const QA: i16 = 255;
pub const QAB: i16 = 255 * 64;

pub const EVAL_SCALE: i16 = 400;



#[repr(C)]
pub struct Network {
    pub feature_weights: [i16; NUM_FEATURES * HIDDEN_SIZE],
    pub feature_biases: [i16; HIDDEN_SIZE],
    pub output_weights: [i16; HIDDEN_SIZE * 2],
    pub output_bias: i16,
}

pub static MODEL: Network = unsafe { mem::transmute(*include_bytes!("./bins/quantised_256.bin")) };
