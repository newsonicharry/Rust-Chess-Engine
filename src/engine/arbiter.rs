use crate::chess::board::Board;
use crate::chess::move_list::MoveList;
use crate::precomputed::accessor::{bishop_lookup, queen_lookup, rook_lookup, MOVEMENT_MASKS};
use crate::chess::types::piece::BasePiece::{Bishop, Knight, Pawn, Queen, Rook};
use crate::engine::types::match_result::MatchResult;
use crate::general::bits;
use std::collections::HashMap;

pub struct Arbiter{}

impl Arbiter{
    pub fn arbitrate(board: &Board, move_list: &MoveList) -> MatchResult {
        if Self::is_checkmate(board, move_list) {
            return MatchResult::Loss
        }

        if Self::is_insufficient_material(board) || Self::is_stalemate(board, move_list) || Self::is_fifty_move_rule(board) || Self::is_three_fold(board) {
            return MatchResult::Draw
        }

        MatchResult::NoResult

    }

    pub fn is_draw(board: &Board) -> bool {
        Self::is_fifty_move_rule(board) || Self::is_insufficient_material(board) || Self::is_three_fold(board)
    }

    fn is_stalemate(board: &Board, move_list: &MoveList) -> bool {
        move_list.move_count() == 0 && !board.in_check()
    }

    fn is_checkmate(board: &Board, move_list: &MoveList) -> bool{
        move_list.move_count() == 0 && board.in_check()
    }

    fn is_fifty_move_rule(board: &Board) -> bool{
        if board.half_move_clock() >= 100 {
            return true;
        }
        false
    }

    // KNNvK is not a draw (as per FIDE rules)
    fn is_insufficient_material(board: &Board) -> bool{
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
        if board.half_move_clock() == 0 {
            return false;
        }

        let mut position_count = HashMap::new();
        position_count.insert(board.zobrist(), 1);

        if let Some(past_states) = board.past_board_states() {
            for past_board_state in past_states.iter().rev() {
                *position_count.entry(past_board_state.zobrist).or_insert(0) += 1;

                if position_count[&past_board_state.zobrist] >= 3 {
                    return true;
                }

                if past_board_state.half_move_clock == 0 {
                    break; // not return false
                }
            }
        }
        false
    }


}