use crate::chess::bitboard::Bitboard;
use crate::chess::board::Board;
use crate::chess::consts::NUM_SQUARES;
use crate::chess::move_list::MoveList;
use crate::chess::precomputed::accessor::{rook_lookup, bishop_lookup, queen_lookup, IN_BETWEEN, MOVEMENT_MASKS};
use crate::chess::types::color::Color;
use crate::chess::types::move_flag::MoveFlag;
use crate::chess::types::piece::BasePiece::{Pawn, Knight, Bishop, Rook, Queen, King};
use crate::chess::types::square::Square;
use crate::chess::types::square::Square::{D1, D7, D8};
use crate::general::bits;

pub struct MoveGenerator{}

impl MoveGenerator {

    pub fn generate(board: &mut Board, move_list: &mut MoveList) {

        board.update_occupancy();
        let (pieces_checking, blockable_squares) = Self::get_check_data(board);
        
        let mut pin_ray_mask: [u64; NUM_SQUARES] = [0; NUM_SQUARES];
        let pinned_pieces_mask = Self::get_pins(board, &mut pin_ray_mask);


        Self::update_pawn_moves  (board, move_list, blockable_squares, pinned_pieces_mask, &pin_ray_mask);
        Self::update_knight_moves(board, move_list, blockable_squares, pinned_pieces_mask);
        Self::update_bishop_moves(board, move_list, blockable_squares, pinned_pieces_mask, &pin_ray_mask);
        Self::update_rook_moves  (board, move_list, blockable_squares, pinned_pieces_mask, &pin_ray_mask);
        Self::update_queen_moves (board, move_list, blockable_squares, pinned_pieces_mask, &pin_ray_mask);

        Self::update_king_moves(board, move_list, pieces_checking);
    }



    fn get_attacks(board: &Board, side_to_move: Color) -> u64{
        let mut attack_mask: u64 = 0;
    
        let all_pieces_no_king = board.all_occupancy() & !board.king_square(!side_to_move).mask();
    
        for &pawn_index in board.color_pieces_of(Pawn, side_to_move) {
            attack_mask |=  MOVEMENT_MASKS.pawn_attacks(side_to_move, pawn_index);
        }
    
        for &knight_index in board.color_pieces_of(Knight, side_to_move)  {
            attack_mask |= MOVEMENT_MASKS.knight[knight_index as usize];
        }
    
        for &bishop_index in board.color_pieces_of(Bishop, side_to_move)  {
            attack_mask |= bishop_lookup(bishop_index, all_pieces_no_king);
        }
    
        for &rook_index in board.color_pieces_of(Rook, side_to_move)  {
            attack_mask |= rook_lookup(rook_index, all_pieces_no_king);
        }
    
        for &queen_index in board.color_pieces_of(Queen, side_to_move)  {
            attack_mask |= queen_lookup(queen_index, all_pieces_no_king);
        }
    
        for &king_index in board.color_pieces_of(King, side_to_move)  {
            attack_mask |= MOVEMENT_MASKS.king[king_index as usize];
        }
    
        attack_mask
    
    }


    fn get_check_data(board: &Board) -> (u64, u64){
    
    
        let king_square = board.king_square(board.side_to_move());

        let enemy_orthogonal = board.color_orthogonal_bitboard(!board.side_to_move());
        let enemy_diagonal = board.color_diagonal_bitboard(!board.side_to_move());
    
        let knight_checks = MOVEMENT_MASKS.knight[king_square as usize] & board.color_bitboard(Knight, !board.side_to_move());
        let pawn_checks = MOVEMENT_MASKS.pawn_attacks(board.side_to_move(), king_square) & board.color_bitboard(Pawn, !board.side_to_move());
    
        let regular_check: u64 = knight_checks | pawn_checks;
        let orthogonal_check: u64 = rook_lookup(king_square, board.all_occupancy()) & enemy_orthogonal;
        let diagonal_check: u64 = bishop_lookup(king_square, board.all_occupancy()) & enemy_diagonal;
    
    
        let all_checks = orthogonal_check | diagonal_check | regular_check;
        let mut blockable_squares: u64 = 0;
    
        if bits::count(all_checks) == 1 {
            let index = bits::next(all_checks);
            blockable_squares = IN_BETWEEN.in_between[king_square as usize][index as usize] | (1 << index)
        }

        // there are no blockable squares (they are all valid so all on)
        if all_checks == 0 {
            blockable_squares = !blockable_squares; 
        }
    
        (all_checks, blockable_squares)
    
    }


    fn update_pawn_moves(board: &Board, move_list: &mut MoveList, blockable_squares: u64, pinned_pieces_mask: u64, pin_ray_mask: &[u64; 64]){


        let king_square = board.king_square(board.side_to_move());

        for &square in board.color_pieces_of(Pawn, board.side_to_move()){

            let mut pawn_attacks = MOVEMENT_MASKS.pawn_attacks(board.side_to_move(), square) & board.color_occupancy(!board.side_to_move()) & blockable_squares;
            let mut pawn_moves = MOVEMENT_MASKS.pawn_move(board.side_to_move(), square) & (!board.all_occupancy()) & blockable_squares;
            let mut double_jump = MOVEMENT_MASKS.pawn_jump(board.side_to_move(), square) & (!board.all_occupancy())  & blockable_squares;


            let pawn_mask = square.mask();
            if pinned_pieces_mask & pawn_mask != 0 {

                double_jump &= pin_ray_mask[square as usize];
                pawn_attacks &= pin_ray_mask[square as usize];
                pawn_moves &= pin_ray_mask[square as usize];
            }

            // en passant
            if let Some(en_passant_file) = board.en_passant_file() {
                if pinned_pieces_mask & pawn_mask == 0 {

                    let en_passant_attack_mask = if board.side_to_move().is_white() {1 << (en_passant_file as u8 + 40)} else { 1<< (en_passant_file as u8 +16) };
                    let attack_square_mask = MOVEMENT_MASKS.pawn_attacks(board.side_to_move(), square) & en_passant_attack_mask;

                    // en passant discovered check
                    if attack_square_mask != 0 {
                        let enemy_pawn_mask: u64 = if board.side_to_move().is_white() {1 << (en_passant_file as u8 + 32)} else { 1<< (en_passant_file as u8 +24) };
                        let new_blockers = board.all_occupancy() & (!enemy_pawn_mask) & (!pawn_mask) | attack_square_mask;

                        let enemy_orthogonal = board.color_orthogonal_bitboard(!board.side_to_move());
                        let enemy_diagonal = board.color_diagonal_bitboard(!board.side_to_move());

                        if rook_lookup(king_square, new_blockers) & enemy_orthogonal == 0{
                            if bishop_lookup(king_square, new_blockers) & enemy_diagonal == 0{
                                move_list.add_moves(attack_square_mask, square, MoveFlag::EnPassantCapture);

                            }
                        }

                    }
                }

            }

            // promotion
            if square.rank().can_pawn_promote(board.side_to_move()) {
                move_list.add_promotion_moves(pawn_moves, square);
                move_list.add_promotion_moves(pawn_attacks, square);

                continue;
            }

            // normal moves and captures
            move_list.add_moves(pawn_attacks, square, MoveFlag::None);
            move_list.add_moves(pawn_moves, square, MoveFlag::None);

            // pawn double jump
            let no_piece_in_way = MOVEMENT_MASKS.pawn_move(board.side_to_move(), square) & (!board.all_occupancy()) != 0;
            if square.rank().is_pawn_start(board.side_to_move()) && no_piece_in_way {
                move_list.add_moves(double_jump, square, MoveFlag::DoubleJump);
            }


        }
    }


    fn update_knight_moves(board: &Board, move_list: &mut MoveList, blockable_squares: u64, pinned_pieces_mask: u64) {

        for &square in board.color_pieces_of(Knight, board.side_to_move()) {

            if pinned_pieces_mask & square.mask() != 0 {
                continue;
            }

            let knight_moves: u64 = (MOVEMENT_MASKS.knight[square as usize]) & !board.color_occupancy(board.side_to_move()) & blockable_squares;

            move_list.add_moves(knight_moves, square, MoveFlag::None);

        }
    }

    fn update_bishop_moves(board: &Board, move_list: &mut MoveList, blockable_squares: u64, pinned_pieces_mask: u64, pin_ray_mask: &[u64; 64]) {

        for &square in  board.color_pieces_of(Bishop, board.side_to_move()){
            let mut bishop_moves: u64 = bishop_lookup(square, board.all_occupancy()) & !board.color_occupancy(board.side_to_move()) & blockable_squares;

            if pinned_pieces_mask & square.mask() != 0 {
                bishop_moves &= pin_ray_mask[square as usize];
            }

            move_list.add_moves(bishop_moves, square, MoveFlag::None);
        }
    }

    fn update_rook_moves(board: &Board, move_list: &mut MoveList, blockable_squares: u64, pinned_pieces_mask: u64, pin_ray_mask: &[u64; 64]) {

        for &square in board.color_pieces_of(Rook, board.side_to_move()) {

            let mut rook_moves: u64 = rook_lookup(square, board.all_occupancy()) & !board.color_occupancy(board.side_to_move()) & blockable_squares;

            if pinned_pieces_mask & square.mask() != 0 {
                rook_moves &= pin_ray_mask[square as usize];
            }

            move_list.add_moves(rook_moves, square, MoveFlag::None);
        }

    }

    fn update_queen_moves(board: &Board, move_list: &mut MoveList, blockable_squares: u64, pinned_pieces_mask: u64, pin_ray_mask: &[u64; 64]) {
        for &square in board.color_pieces_of(Queen, board.side_to_move()){

            let mut queen_moves: u64 = queen_lookup(square, board.all_occupancy()) & !board.color_occupancy(board.side_to_move()) & blockable_squares;

            if pinned_pieces_mask & square.mask() != 0 {
                queen_moves &= pin_ray_mask[square as usize];
            }

            move_list.add_moves(queen_moves, square, MoveFlag::None);
        }

    }

    fn update_king_moves(board: &Board, move_list: &mut MoveList, pieces_checking: u64) {

        let attack_squares =  Self::get_attacks(board, !board.side_to_move());
        let king_square = board.king_square(board.side_to_move());

        let valid_moves: u64 = (MOVEMENT_MASKS.king[king_square as usize]) & !board.color_occupancy(board.side_to_move()) & !attack_squares;

        move_list.add_moves(valid_moves, king_square, MoveFlag::None);

        if board.has_short_castle_rights(board.side_to_move()) && pieces_checking == 0{
            let clear_squares = if board.side_to_move().is_white() { 96 } else { 6917529027641081856 };

            if (clear_squares & attack_squares == 0) && (clear_squares & board.all_occupancy() == 0) {

                let move_to_square = if board.side_to_move().is_white() { Square::from(6) } else { Square::from(62) };
                move_list.add_moves(move_to_square.mask(), king_square, MoveFlag::CastleShort);

            }
        }

        if board.has_long_castle_rights(board.side_to_move()) && pieces_checking == 0{
            let not_attacked_squares: u64 = if board.side_to_move().is_white() { 12 } else { 864691128455135232 };
            let not_occupied_squares: u64 = if board.side_to_move().is_white() { 14 } else { 1008806316530991104 };

            if (not_attacked_squares & attack_squares == 0) && (not_occupied_squares & board.all_occupancy() == 0) {
                let move_to_index: u8 = if board.side_to_move().is_white() { 2 } else { 58 };

                move_list.add_moves(1 << move_to_index, king_square, MoveFlag::CastleLong);
            }

        }

    }

    
    
    fn get_pins(board: &Board, pin_ray_mask: &mut [u64; 64]) -> u64{
    
    
        let friendly_king_square = board.king_square(board.side_to_move());
        let friendly_pieces = board.color_occupancy(board.side_to_move());
        let enemy_pieces = board.color_occupancy(!board.side_to_move());

        let enemy_orthogonal = board.color_orthogonal_bitboard(!board.side_to_move());
        let enemy_diagonal = board.color_diagonal_bitboard(!board.side_to_move());

        // println!("{}", Bitboard::from(enemy_orthogonal));
        // println!("{}", Bitboard::from(MOVEMENT_MASKS.rook[friendly_king_square as usize]));

        let possible_orthogonally_pinned: u64 = MOVEMENT_MASKS.rook[friendly_king_square as usize] & enemy_orthogonal;
        let possible_diagonally_pinned: u64 = MOVEMENT_MASKS.bishop[friendly_king_square as usize] & enemy_diagonal;

        let mut possible_pinners = possible_orthogonally_pinned | possible_diagonally_pinned;



        let mut pinned_pieces_mask: u64 = 0;
    
        while possible_pinners != 0{
            let possible_pinner = bits::next(possible_pinners);
            let ray = IN_BETWEEN.in_between[friendly_king_square as usize][possible_pinner as usize];
    
            // opponents between the king and pinner
            if (ray & enemy_pieces) != 0 {
                possible_pinners = bits::pop(possible_pinners);
                continue;
            }
    
            let friendly_pieces_between = ray & friendly_pieces;
    
            if bits::count(friendly_pieces_between) == 1 {
                pinned_pieces_mask |= friendly_pieces_between;
                let friendly_piece_index = bits::next(friendly_pieces_between) as usize;
                pin_ray_mask[friendly_piece_index] = IN_BETWEEN.in_between[friendly_king_square as usize][possible_pinner as usize] | (1 << possible_pinner);
            }
    
            possible_pinners = bits::pop(possible_pinners);
        }
    
    
        pinned_pieces_mask
    
    }
}