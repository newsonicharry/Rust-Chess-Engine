use std::mem::MaybeUninit;

use crate::chess::{
    board::Board,
    consts::{MAX_DEPTH, MAX_MOVES, NUM_PIECES, NUM_SQUARES},
    move_ply::MovePly,
    types::{color::Color, piece::Piece, square::Square},
};

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
        if self.data[depth as usize][0] == played {
            return;
        }

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

type Ply1 = [[[[i16; NUM_PIECES]; NUM_SQUARES]; NUM_PIECES]; NUM_SQUARES];
// type Ply2 = [[Ply1; NUM_PIECES]; NUM_SQUARES];
// type Ply3 = [[Ply2; NUM_PIECES]; NUM_SQUARES];
// type Ply4 = [[Ply3; NUM_PIECES]; NUM_SQUARES];

pub struct ContinuationHistory {
    ply_1_table: Ply1,
    // ply_2_table: Ply2,
    // ply_4_table: Ply4,
}

impl Default for ContinuationHistory {
    fn default() -> Self {
        Self {
            ply_1_table: unsafe { MaybeUninit::zeroed().assume_init() },
            // ply_2_table: unsafe { MaybeUninit::zeroed().assume_init() },
            // ply_4_table: unsafe { MaybeUninit::zeroed().assume_init() },
        }
    }
}

impl ContinuationHistory {
    pub fn update(
        &mut self,
        other_quiets: &Vec<MovePly>,
        depth: u8,
        ply: u8,
        board: &Board,
        search_stack: &[(MovePly, Piece); MAX_MOVES],
    ) {
        let clamped_update = |entry: &mut i16, bonus: i16| {
            *entry += bonus - (*entry as i32 * bonus.abs() as i32 / MAX_HISTORY as i32) as i16;
        };

        let (cutoff_move, cutoff_piece) = search_stack[ply as usize];

        let ply_1 = ply as i16 - 1;
        if ply_1.is_negative() {
            return;
        }

        let cutoff_to = cutoff_move.to() as usize;
        let cutoff_piece = cutoff_piece as usize;
        let ply_1_to = search_stack[ply_1 as usize].0.to() as usize;
        let ply_1_piece = search_stack[ply_1 as usize].1 as usize;

        if ply_1_piece >= NUM_PIECES {
            return;
        }

        let entry = &mut self.ply_1_table[cutoff_to][cutoff_piece][ply_1_to][ply_1_piece];
        let bonus = (depth as i16).pow(2).min(400);

        clamped_update(entry, bonus);

        for quiet in other_quiets {
            let quiet_to = quiet.to() as usize;
            let quiet_piece = board.piece_at(quiet.from()) as usize;
            let entry = &mut self.ply_1_table[quiet_to][quiet_piece][ply_1_to][ply_1_piece];
            clamped_update(entry, -bonus);
        }
    }

    pub fn get_ply_1(
        &self,
        to: Square,
        piece: Piece,
        ply: u8,
        search_stack: &[(MovePly, Piece); MAX_MOVES],
    ) -> Option<i16> {
        if ply < 1 {
            return None;
        }

        let ply_1 = (ply - 1) as usize;
        let next_to = search_stack[ply_1].0.to() as usize;
        let next_piece = search_stack[ply_1].1 as usize;

        if next_piece >= NUM_PIECES {
            return None;
        }

        Some(self.ply_1_table[to as usize][piece as usize][next_to][next_piece])
    }
}
