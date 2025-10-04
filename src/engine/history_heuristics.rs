use crate::chess::consts::NUM_SQUARES;
use crate::chess::move_ply::MovePly;
use crate::chess::types::color::Color;

const SQUARE_MOVE_MASK: u16 = 0xFFF;

pub struct HistoryHeuristics {
    square_data: [[u16; NUM_SQUARES*NUM_SQUARES]; 2],
}

impl Default for HistoryHeuristics {
    fn default() -> HistoryHeuristics {
        HistoryHeuristics{
            square_data: [[0; NUM_SQUARES*NUM_SQUARES]; 2]
        }
    }
}

impl HistoryHeuristics {

    pub fn get_history(&self, cur_move: MovePly, side_to_move: Color) -> u16{
        let cur_square_data = (cur_move.packed_data() & SQUARE_MOVE_MASK) as usize;
        self.square_data[side_to_move as usize][cur_square_data]
    }

    pub fn update_history(&mut self, cur_move: MovePly, side_to_move: Color){
        let cur_square_data = (cur_move.packed_data() & SQUARE_MOVE_MASK) as usize;
        self.square_data[side_to_move as usize][cur_square_data] += 1;
    }
}