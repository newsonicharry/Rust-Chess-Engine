use std::ffi::c_int;
use crate::chess::board::Board;
use crate::chess::consts::MAX_DEPTH;
use crate::chess::move_ply::MovePly;

pub struct Killers{
    moves: [[MovePly; 2]; MAX_DEPTH]
}

impl Default for Killers {
    fn default() -> Self {
        Killers{
            moves: [[MovePly::default(); 2]; MAX_DEPTH]
        }
    }
}

impl Killers {


    pub fn contains(&self, depth: u8, checked_move: MovePly) -> bool{
        if self.first_occupancy(depth) == checked_move || self.second_occupancy(depth) == checked_move {
            return true;
        }

        false
    }

    fn first_occupancy(&self, depth: u8) -> MovePly{
        self.moves[depth as usize][0]
    }

    fn second_occupancy(&self, depth: u8) -> MovePly{
        self.moves[depth as usize][1]
    }

    pub fn update(&mut self, cur_move: MovePly, depth: u8){

        let first_occupancy = self.first_occupancy(depth);
        let second_occupancy = self.second_occupancy(depth);

        if first_occupancy.is_default() {
            self.moves[depth as usize][0] = cur_move;
        }
        else if second_occupancy.is_default() {
            self.moves[depth as usize][1] = cur_move;
        }
        else {
            self.moves[depth as usize][0] = cur_move;
            self.moves[depth as usize][1] = first_occupancy;
        }

    }
}


