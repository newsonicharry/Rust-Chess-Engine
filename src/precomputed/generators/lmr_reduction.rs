use crate::chess::consts::{MAX_DEPTH, MAX_POSITIONAL_MOVES};

pub struct LMRReduction{
    reduction_table: [[u8; MAX_POSITIONAL_MOVES]; MAX_DEPTH]
}


impl LMRReduction {

    pub fn new() -> LMRReduction {
        let mut reduction_table = [[0; MAX_POSITIONAL_MOVES]; MAX_DEPTH];

        for depth in 0..MAX_DEPTH {
            for move_index in 0..MAX_POSITIONAL_MOVES {
                let mut lmr = 0.75 + (depth as f32).ln() * (move_index as f32).ln() / 2f32;
                lmr = lmr.max(0f32).min(depth as f32);

                reduction_table[depth][move_index] = lmr as u8;
            }
        }


        LMRReduction{ reduction_table }
    }
    pub fn reduction(&self, depth: u8, move_order: u8) -> u8{
        self.reduction_table[depth as usize][move_order as usize]
    }
}