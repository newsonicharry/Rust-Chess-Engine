use std::slice::Iter;
use rand::seq::SliceRandom;
use rand::thread_rng;
use crate::chess::move_ply::MovePly;
use crate::chess::types::move_flag::MoveFlag;
use crate::chess::types::square::Square;
use crate::general::bits;

#[derive(Clone, Copy)]
pub struct MoveList{ 
    moves: [MovePly; 256],
    move_count: usize,
    index: usize,
}


impl Default for MoveList {
    fn default() -> Self {
        MoveList{
            moves: unsafe { core::mem::MaybeUninit::zeroed().assume_init() },
            move_count: 0,
            index: 0,
        }
    }
}

impl MoveList {
    pub fn add_moves(&mut self, mut to_mask: u64, from: Square, move_flag: MoveFlag) {
        while to_mask != 0 {
            let to = Square::from(bits::next(to_mask));

            self.moves[self.move_count] = MovePly::new(from, to, move_flag);
            to_mask &= to_mask - 1;

            self.move_count += 1;
        }
    }

    pub fn add_promotion_moves(&mut self, mut to_mask: u64, from: Square) {
        while to_mask != 0 {
            let to_square = Square::from(bits::next(to_mask));

            self.moves[self.move_count] = MovePly::new(from, to_square, MoveFlag::PromoteToKnight);
            self.moves[self.move_count + 1] = MovePly::new(from, to_square, MoveFlag::PromoteToBishop);
            self.moves[self.move_count + 2] = MovePly::new(from, to_square, MoveFlag::PromoteToRook);
            self.moves[self.move_count + 3] = MovePly::new(from, to_square, MoveFlag::PromoteToQueen);

            self.move_count += 4;

            to_mask = bits::pop(to_mask);
        }
    }

    pub fn reset(&mut self) {
        self.move_count = 0;
    }

    pub fn move_count(&self) -> usize {
        self.move_count
    }

    pub fn move_at(&self, index: usize) -> MovePly {
        self.moves[index]
    }

    pub fn iter(&mut self) -> Iter<'_, MovePly> {

        self.moves[..self.move_count].iter()

        
    }


}
