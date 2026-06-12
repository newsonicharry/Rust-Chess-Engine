use crate::chess::consts::NUM_SQUARES;
use crate::chess::move_ply::MovePly;
use crate::chess::types::color::Color;

const SQUARE_MOVE_MASK: u16 = 0xFFF;

pub struct HistoryHeuristics {
    square_data: [[i16; NUM_SQUARES * NUM_SQUARES]; 2],
}

impl Default for HistoryHeuristics {
    fn default() -> HistoryHeuristics {
        HistoryHeuristics {
            square_data: [[0; NUM_SQUARES * NUM_SQUARES]; 2],
        }
    }
}

impl HistoryHeuristics {
    fn bonus(depth: u8) -> i16 {
        (depth as i16 * depth as i16).min(1200)
    }

    fn update_value(&mut self, value: i16, cur_move: &MovePly, side_to_move: Color) {
        let old_history = &mut self.square_data[side_to_move as usize]
            [(cur_move.packed_data() & SQUARE_MOVE_MASK) as usize];

        *old_history = old_history.saturating_add(value);
    }

    pub fn get(&self, cur_move: MovePly, side_to_move: Color) -> i16 {
        let cur_square_data = (cur_move.packed_data() & SQUARE_MOVE_MASK) as usize;
        self.square_data[side_to_move as usize][cur_square_data]
    }

    pub fn update(
        &mut self,
        best_move: &MovePly,
        other_quiets: &Vec<MovePly>,
        side_to_move: Color,
        depth: u8,
    ) {
        let square_data = (best_move.packed_data() & SQUARE_MOVE_MASK) as usize;
        self.square_data[side_to_move as usize][square_data] += Self::bonus(depth);
        // self.square_data[side_to_move as usize][square_data] += 1;

        for quiet in other_quiets {
            let square_data = (quiet.packed_data() & SQUARE_MOVE_MASK) as usize;
            // self.square_data[side_to_move as usize][square_data] -= Self::bonus(depth) / 4;
        }
    }

    // pub fn penalize(&mut self, other_moves: , side_to_move: Color) {
    //     let cur_square_data = (cur_move.packed_data() & SQUARE_MOVE_MASK) as usize;
    //     let new_value = self.square_data[side_to_move as usize][cur_square_data].saturating_sub(1);
    //     self.square_data[side_to_move as usize][cur_square_data] = new_value;
    // }
}
