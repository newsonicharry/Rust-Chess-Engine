use crate::chess::move_ply::MovePly;
use crate::chess::types::move_flag::MoveFlag;
use crate::chess::types::square::Square;
use crate::general::bits;

#[derive(Clone, Copy)]
pub struct MoveList{ 
    moves: [MovePly; 256],
    move_count: usize,
}


impl Default for MoveList {
    fn default() -> Self {
        MoveList{
            moves: [MovePly::default(); 256],
            move_count: 0,
        }
    }
}

impl MoveList{

    pub fn add_moves(&mut self, mut to_mask: u64, from: Square, move_flag: MoveFlag){
        while to_mask != 0{

            let to = Square::from(bits::next(to_mask));

            self.moves[self.move_count] = MovePly::new(from, to, move_flag);
            to_mask &= to_mask - 1;

            self.move_count += 1;
        }
    }

    pub fn reset(&mut self){
        self.move_count = 0;
    }

    pub fn move_count(&self) -> usize{
        self.move_count
    }

    pub fn move_at(&self, index: usize) -> MovePly{
        self.moves[index]
    }

}