use crate::chess::move_ply::MovePly;

const SQUARE_MOVE_MASK: u16 = 0xFFF;

pub struct HistoryHeuristics {
    square_data: [u16; 4096]
}

impl Default for HistoryHeuristics {
    fn default() -> HistoryHeuristics {
        HistoryHeuristics{
            square_data: [0; 4096]
        }
    }
}

impl HistoryHeuristics {

    pub fn get_history(&self, cur_move: MovePly) -> u16{
        let cur_square_data = (cur_move.packed_data() & SQUARE_MOVE_MASK) as usize;
        self.square_data[cur_square_data]
    }

    pub fn update_history(&mut self, cur_move: MovePly){
        let cur_square_data = (cur_move.packed_data() & SQUARE_MOVE_MASK) as usize;
        self.square_data[cur_square_data] += 1;
    }
}