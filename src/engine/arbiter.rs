use crate::chess::board::Board;
use crate::chess::types::color::Color::White;
use crate::chess::types::piece::BasePiece::{Bishop, Knight, Pawn, Queen, Rook};
use crate::engine::types::match_result::MatchResult;
use crate::general::bits;
use crate::chess::move_list::MoveList;
use crate::chess::precomputed::accessor::{MOVEMENT_MASKS, rook_lookup, bishop_lookup, queen_lookup};
use std::collections::HashMap;

pub struct Arbiter{}

impl Arbiter{
    pub fn arbitrate(board: &Board, move_list: &MoveList) -> MatchResult {
        if Self::is_checkmate(board, move_list) {
            return MatchResult::Loss
        }

        if Self::is_insufficient_material(board) || Self::is_stalemate(move_list) {
            return MatchResult::Draw
        }

        MatchResult::NoResult

    }

    fn is_stalemate(move_list: &MoveList) -> bool {
        move_list.move_count() == 0
    }

    fn is_checkmate(board: &Board, move_list: &MoveList) -> bool{
        if move_list.move_count() == 0{
            let king_square = board.king_square(board.side_to_move());

            let rooks_checking_king = rook_lookup(king_square, board.all_occupancy()) & board.color_bitboard(Rook, !board.side_to_move());
            if rooks_checking_king != 0 { return true }

            let queens_checking_king = queen_lookup(king_square, board.all_occupancy()) & board.color_bitboard(Queen, !board.side_to_move());
            if queens_checking_king != 0 { return true }

            let knights_checking_king = MOVEMENT_MASKS.knight[king_square as usize] & board.color_bitboard(Knight, !board.side_to_move());
            if knights_checking_king != 0 { return true }

            let bishops_checking_king = bishop_lookup(king_square, board.all_occupancy()) & board.color_bitboard(Bishop, !board.side_to_move());
            if bishops_checking_king != 0 { return true }

            let pawns_checking_king = MOVEMENT_MASKS.pawn_attacks(board.side_to_move(), king_square) & board.color_bitboard(Pawn, !board.side_to_move());
            if pawns_checking_king != 0 { return true }

            return false
        }
        false
    }

    fn is_fifty_move_rule(board: &Board) -> bool{
        if board.half_move_clock() >= 100 {
            return true;
        }
        false
    }

    // KNNvK is not a draw (as per FIDE rules)
    fn is_insufficient_material(board: &Board) -> bool{
        if bits::count(board.all_occupancy()) <= 3 {

            if bits::count(board.combined_bitboard(Pawn)) != 0 {
                return false;
            }

            if bits::count(board.combined_bitboard(Rook)) != 0 {
                return false;
            }

            if bits::count(board.combined_bitboard(Queen)) != 0 {
                return false;
            }

            return true;
        }

        false
    }
    
    fn is_three_fold(board: &Board) -> bool{

        // if the half move clock is zero it means the position changed and cannot be repeated
        if board.half_move_clock() == 0 {
            return false;
        }

        let mut position_count = HashMap::new();
        position_count.insert(board.zobrist(), 1);

        let past_states = board.past_board_states();
        if past_states.is_none() {
            return false;
        }

        for past_board_state in past_states.unwrap().iter().rev() {
            *position_count.entry(past_board_state.zobrist).or_insert(1) += 1;

            if *position_count.entry(past_board_state.zobrist).key() == 3 {
                return true;
            }

            if past_board_state.half_move_clock == 0 {
                return false;
            }
        }

        false
    }
}