use crate::chess::board::Board;
use crate::chess::consts::{self, MAX_DEPTH};
use crate::chess::move_ply::MovePly;
use crate::chess::types::piece::{BasePiece, Piece};
use crate::chess::types::square::Square;

pub struct CaptureHeuristics {
    values: [[[i16; consts::NUM_BASE_PIECES]; consts::NUM_PIECES]; consts::NUM_SQUARES],
}

impl Default for CaptureHeuristics {
    fn default() -> Self {
        Self {
            values: [[[0; consts::NUM_BASE_PIECES]; consts::NUM_PIECES]; consts::NUM_SQUARES],
        }
    }
}

impl CaptureHeuristics {
    fn bonus(depth: u8) -> i16 {
        (depth as i16 * depth as i16).min(1200)
    }

    fn update_value(
        &mut self,
        value: i16,
        target: Square,
        capturing_piece: Piece,
        captured_piece: BasePiece,
    ) {
        let old_history =
            &mut self.values[target as usize][capturing_piece as usize][captured_piece as usize];

        *old_history = old_history.saturating_add(value);
    }

    pub fn update(
        &mut self,
        board: &Board,
        best_move: &MovePly,
        other_captures: &Vec<MovePly>,
        depth: u8,
    ) {
        for capture in other_captures {
            let capturing_piece = board.piece_at(capture.from());
            let captured_piece = board.piece_at(capture.to()).into();

            let penalty = -Self::bonus(depth) / 2;

            self.update_value(penalty, capture.to(), capturing_piece, captured_piece);
        }

        let capturing_piece = board.piece_at(best_move.from());
        let captured_piece = board.piece_at(best_move.to()).into();

        let bonus = Self::bonus(depth);

        self.update_value(bonus, best_move.to(), capturing_piece, captured_piece);
    }

    pub fn get(&self, target: Square, capturing_piece: Piece, captured_piece: Piece) -> i16 {
        let captured_piece: BasePiece = captured_piece.into();

        self.values[target as usize][capturing_piece as usize][captured_piece as usize]
    }
}
