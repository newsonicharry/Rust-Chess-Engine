use crate::chess::consts::NUM_SQUARES;
use crate::chess::move_ply::MovePly;
use crate::chess::types::color::Color;

const SQUARE_MOVE_MASK: u16 = 0xFFF;

pub struct CounterMoveHeuristics {
    square_data: [[MovePly; NUM_SQUARES*NUM_SQUARES]; 2],
}

impl Default for CounterMoveHeuristics {
    fn default() -> CounterMoveHeuristics {
        CounterMoveHeuristics {
            square_data: [[MovePly::default(); NUM_SQUARES*NUM_SQUARES]; 2]
        }
    }
}

impl CounterMoveHeuristics {

    pub fn get_counter_move(&self, cur_move: MovePly, side_to_move: Color) -> MovePly{
        let cur_square_data = (cur_move.packed_data() & SQUARE_MOVE_MASK) as usize;
        let counter_move = self.square_data[side_to_move as usize][cur_square_data];
        
        counter_move
    }

    pub fn update_counter_move(&mut self, cur_move: MovePly, counter_move: MovePly, side_to_move: Color){
        let cur_square_data = (cur_move.packed_data() & SQUARE_MOVE_MASK) as usize;
        self.square_data[side_to_move as usize][cur_square_data] = counter_move;

    }
}