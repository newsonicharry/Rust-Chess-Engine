use crate::chess::board::Board;
use crate::chess::consts::{NUM_PIECES, NUM_SQUARES};
use crate::chess::types::color::Color;
use crate::chess::types::piece::Piece;
use crate::chess::types::square::Square;
use crate::chess::{consts::MAX_DEPTH, move_ply::MovePly};

pub struct KillerMoves {
    data: [[MovePly; 2]; MAX_DEPTH],
}

impl Default for KillerMoves {
    fn default() -> Self {
        Self {
            data: [[MovePly::default(); 2]; MAX_DEPTH],
        }
    }
}

impl KillerMoves {
    pub fn update(&mut self, played: MovePly, depth: u8) {
        self.data[depth as usize][1] = self.data[depth as usize][0];
        self.data[depth as usize][0] = played;
    }

    pub fn is_first_slot(&self, played: MovePly, depth: u8) -> bool {
        self.data[depth as usize][0] == played
    }

    pub fn is_second_slot(&self, played: MovePly, depth: u8) -> bool {
        self.data[depth as usize][1] == played
    }
}

const SQUARE_MOVE_MASK: u16 = 0xFFF;
const MAX_HISTORY: i16 = 2i16.pow(14);

pub struct CounterMoveHeuristics {
    table: [[i16; NUM_SQUARES * NUM_SQUARES]; 2],
}

impl Default for CounterMoveHeuristics {
    fn default() -> CounterMoveHeuristics {
        CounterMoveHeuristics {
            table: [[0; NUM_SQUARES * NUM_SQUARES]; 2],
        }
    }
}

#[allow(dead_code)]
impl CounterMoveHeuristics {
    fn clamped_update(&mut self, bonus: i16, curr_square_data: usize, color: Color) {
        let curr_history = self.table[color as usize][curr_square_data as usize];
        self.table[color as usize][curr_square_data as usize] +=
            bonus - (curr_history as i32 * bonus.abs() as i32 / MAX_HISTORY as i32) as i16;
    }

    pub fn update(
        &mut self,
        best_move: MovePly,
        other_quiets: &Vec<MovePly>,
        depth: u8,
        curr_color: Color,
    ) {
        let bonus = (depth as i16).pow(2).min(400);
        let get_square_data = |mv: MovePly| (mv.packed_data() & SQUARE_MOVE_MASK) as usize;

        self.clamped_update(bonus, get_square_data(best_move), curr_color);

        for quiet in other_quiets {
            self.clamped_update(-bonus, get_square_data(*quiet), curr_color);
        }
    }

    pub fn get(&self, cur_move: MovePly, side_to_move: Color) -> i16 {
        let curr_square_data = (cur_move.packed_data() & SQUARE_MOVE_MASK) as usize;
        self.table[side_to_move as usize][curr_square_data]
    }
}

pub struct History {
    table: [[i16; NUM_SQUARES]; NUM_PIECES],
}

impl Default for History {
    fn default() -> Self {
        Self {
            table: [[0; NUM_SQUARES]; NUM_PIECES],
        }
    }
}

impl History {
    fn clamped_update(&mut self, bonus: i16, piece: Piece, square: Square) {
        let curr_history = self.table[piece as usize][square as usize];
        self.table[piece as usize][square as usize] +=
            bonus - (curr_history as i32 * bonus.abs() as i32 / MAX_HISTORY as i32) as i16;
    }

    pub fn update(
        &mut self,
        best_move: MovePly,
        other_quiets: &Vec<MovePly>,
        depth: u8,
        board: &Board,
    ) {
        let bonus = (depth as i16).pow(2).min(400);

        self.clamped_update(bonus, board.piece_at(best_move.from()), best_move.to());

        for quiet in other_quiets {
            self.clamped_update(-bonus, board.piece_at(quiet.from()), quiet.to());
        }
    }

    pub fn get(&self, piece: Piece, square: Square) -> i16 {
        self.table[piece as usize][square as usize]
    }
}
