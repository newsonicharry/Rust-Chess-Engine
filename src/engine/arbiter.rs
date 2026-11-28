use crate::{
    chess::{
        board::Board,
        move_list::MoveList,
        types::piece::BasePiece::{Pawn, Queen, Rook},
    },
    general::bits,
};

#[derive(Debug)]
pub enum MatchResult {
    Loss,
    Draw,
    NoResult,
}

pub struct Arbiter {}

impl Arbiter {
    pub fn arbitrate(board: &Board, move_list: &MoveList) -> MatchResult {
        if Self::is_checkmate(board, move_list) {
            return MatchResult::Loss;
        }
        if Self::is_insufficient_material(board)
            || Self::is_stalemate(board, move_list)
            || Self::is_fifty_move_rule(board)
            || board.is_repetition()
        {
            return MatchResult::Draw;
        }

        MatchResult::NoResult
    }

    fn is_stalemate(board: &Board, move_list: &MoveList) -> bool {
        move_list.move_count() == 0 && !board.in_check()
    }

    fn is_checkmate(board: &Board, move_list: &MoveList) -> bool {
        move_list.move_count() == 0 && board.in_check()
    }

    fn is_fifty_move_rule(board: &Board) -> bool {
        if board.half_move_clock() >= 100 {
            return true;
        }
        false
    }

    // KNNvK is not a draw (as per FIDE rules)
    fn is_insufficient_material(board: &Board) -> bool {
        if bits::count(board.occupancy()) <= 3 {
            if bits::count(board.bitboard_combined(Pawn)) != 0 {
                return false;
            }

            if bits::count(board.bitboard_combined(Rook)) != 0 {
                return false;
            }

            if bits::count(board.bitboard_combined(Queen)) != 0 {
                return false;
            }

            return true;
        }

        false
    }

    fn is_three_fold(board: &Board) -> bool {
        // if board.half_move_clock() == 0 {
        //     return false;
        // }

        if let Some(past_states) = board.past_board_states() {
            for past_state in past_states.iter().rev() {
                if past_state.zobrist == board.zobrist() {
                    return true;
                }

                if past_state.half_move_clock == 0 {
                    break;
                }
            }
        }
        false
    }
}
