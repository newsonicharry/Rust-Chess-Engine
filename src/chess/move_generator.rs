use crate::chess::board::Board;
use crate::chess::consts::NUM_SQUARES;
use crate::chess::move_list::MoveList;
use crate::precomputed::accessor::{bishop_lookup, queen_lookup, rook_lookup, slider_lookup, IN_BETWEEN, MOVEMENT_MASKS};
use crate::chess::types::move_flag::MoveFlag;
use crate::chess::types::piece::BasePiece;
use crate::chess::types::piece::BasePiece::{Bishop, King, Knight, Pawn, Queen, Rook};
use crate::chess::types::square::Square;
use crate::general::bits;

pub struct MoveGenerator<const GENERATOR_TYPE: bool>{}

pub const GEN_ALL: bool = false;
pub const GEN_TACTICS: bool = true;


impl<const GENERATOR_TYPE: bool> MoveGenerator<GENERATOR_TYPE> {

    pub fn generate(board: &mut Board, move_list: &mut MoveList) {
        board.update_occupancy();
        let (pieces_checking, allowed_squares) = Self::get_check_data(board);

        let mut pin_ray_mask: [u64; NUM_SQUARES] = [u64::MAX; NUM_SQUARES];
        let pinned_pieces_mask = Self::get_pins(board, &mut pin_ray_mask);

        if pieces_checking != 0 { board.set_in_check(true); }
        else { board.set_in_check(false); }

        Self::update_pawn_moves(board, move_list, allowed_squares, &pin_ray_mask);
        Self::update_knight_moves(board, move_list, allowed_squares, pinned_pieces_mask);

        Self::update_king_moves(board, move_list, pieces_checking);

        Self::update_slider_moves(Bishop, board, move_list, allowed_squares, &pin_ray_mask);
        Self::update_slider_moves(Rook, board, move_list, allowed_squares, &pin_ray_mask);
        Self::update_slider_moves(Queen, board, move_list, allowed_squares, &pin_ray_mask);
    }



    fn get_enemy_attacks(board: &Board) -> u64{
        let mut attack_mask: u64 = 0;
    
        let all_pieces_no_king = board.occupancy() & !board.king_square(board.side_to_move()).mask();
    
        for &pawn_square in board.piece_list_them(Pawn){
            attack_mask |=  MOVEMENT_MASKS.pawn_attacks(!board.side_to_move(), pawn_square);
        }
    
        for &knight_square in board.piece_list_them(Knight)  {
            attack_mask |= MOVEMENT_MASKS.knight[knight_square as usize];
        }
    
        for &bishop_square in board.piece_list_them(Bishop)  {
            attack_mask |= bishop_lookup(bishop_square, all_pieces_no_king);
        }
    
        for &rook_square in board.piece_list_them(Rook)  {
            attack_mask |= rook_lookup(rook_square, all_pieces_no_king);
        }
    
        for &queen_square in board.piece_list_them(Queen)  {
            attack_mask |= queen_lookup(queen_square, all_pieces_no_king);
        }
    
        for &king_square in board.piece_list_them(King)  {
            attack_mask |= MOVEMENT_MASKS.king[king_square as usize];
        }
    
        attack_mask
    
    }


    fn get_check_data(board: &Board) -> (u64, u64){
    
    
        let king_square = board.king_square(board.side_to_move());

        let enemy_orthogonal = board.orthogonal_bitboard_them();
        let enemy_diagonal = board.diagonal_bitboard_them();
    
        let knight_checks = MOVEMENT_MASKS.knight[king_square as usize] & board.bitboard_them(Knight);
        let pawn_checks = MOVEMENT_MASKS.pawn_attacks(board.side_to_move(), king_square) & board.bitboard_them(Pawn);
    
        let regular_check: u64 = knight_checks | pawn_checks;
        let orthogonal_check: u64 = rook_lookup(king_square, board.occupancy()) & enemy_orthogonal;
        let diagonal_check: u64 = bishop_lookup(king_square, board.occupancy()) & enemy_diagonal;
    
    
        let all_checks = orthogonal_check | diagonal_check | regular_check;
        let mut allowed_squares: u64 = 0;
    
        if bits::count(all_checks) == 1 {
            let index = bits::next(all_checks);
            allowed_squares = IN_BETWEEN.in_between[king_square as usize][index as usize] | (1 << index)
        }

        // there are no blockable squares (they are all valid so all on)
        if all_checks == 0 {
            allowed_squares = !allowed_squares;
        }
    
        (all_checks, allowed_squares)
    
    }


    fn update_pawn_moves(board: &Board, move_list: &mut MoveList, allowed_squares: u64, pin_ray_mask: &[u64; 64]){
        
        let king_square = board.king_square(board.side_to_move());

        for &square in board.piece_list_us(Pawn){

            let pin_mask = pin_ray_mask[square as usize];
            let pawn_attacks = MOVEMENT_MASKS.pawn_attacks(board.side_to_move(), square) & board.occupancy_them() & allowed_squares & pin_mask;
            let pawn_moves = MOVEMENT_MASKS.pawn_move(board.side_to_move(), square) & (!board.occupancy()) & allowed_squares & pin_mask;
            let double_jump = MOVEMENT_MASKS.pawn_jump(board.side_to_move(), square) & (!board.occupancy())  & allowed_squares & pin_mask;

            let pawn_mask = square.mask();

            // en passant
            if let Some(en_passant_file) = board.en_passant_file() {
                if pin_mask == u64::MAX  {

                    let en_passant_attack_mask = if board.side_to_move().is_white() {1 << (en_passant_file as u8 + 40)} else { 1<< (en_passant_file as u8 +16) };
                    let attack_square_mask = MOVEMENT_MASKS.pawn_attacks(board.side_to_move(), square) & en_passant_attack_mask;

                    // en passant discovered check
                    if attack_square_mask != 0 {
                        let enemy_pawn_mask: u64 = if board.side_to_move().is_white() {1 << (en_passant_file as u8 + 32)} else { 1<< (en_passant_file as u8 +24) };
                        let new_blockers = board.occupancy() & (!enemy_pawn_mask) & (!pawn_mask) | attack_square_mask;

                        let enemy_orthogonal = board.orthogonal_bitboard_them();
                        let enemy_diagonal = board.diagonal_bitboard_them();

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

            move_list.add_moves(pawn_attacks, square, MoveFlag::None);
            
            if GENERATOR_TYPE == GEN_ALL {
                // normal moves and captures
                move_list.add_moves(pawn_moves, square, MoveFlag::None);

                // pawn double jump
                let no_piece_in_way = MOVEMENT_MASKS.pawn_move(board.side_to_move(), square) & (!board.occupancy()) != 0;
                if square.rank().is_pawn_start(board.side_to_move()) && no_piece_in_way {
                    move_list.add_moves(double_jump, square, MoveFlag::DoubleJump);
                }
            }
            


        }
    }


    fn update_knight_moves(board: &Board, move_list: &mut MoveList, mut allowed_squares: u64, pinned_pieces_mask: u64) {

        if GENERATOR_TYPE == GEN_TACTICS {
            let enemy_king = board.king_square(!board.side_to_move());
            let checking_squares = MOVEMENT_MASKS.knight[enemy_king as usize];

            allowed_squares = (allowed_squares & board.occupancy_them()) | (allowed_squares & checking_squares);
        }
        
        for &square in board.piece_list_us(Knight) {

            if pinned_pieces_mask & square.mask() != 0 {
                continue;
            }

            let knight_moves: u64 = (MOVEMENT_MASKS.knight[square as usize]) & !board.occupancy_us() & allowed_squares;

            move_list.add_moves(knight_moves, square, MoveFlag::None);

        }
    }

    fn update_slider_moves(slider_type: BasePiece, board: &Board, move_list: &mut MoveList, mut allowed_squares: u64, pin_ray_mask: &[u64; 64]) {
        if GENERATOR_TYPE == GEN_TACTICS {
            let enemy_king = board.king_square(!board.side_to_move());
            let checking_squares = slider_lookup(slider_type, enemy_king, board.occupancy());

            allowed_squares = (allowed_squares & board.occupancy_them()) | (allowed_squares & checking_squares);
        }
        
        for &square in  board.piece_list_us(slider_type){
            let slider_moves: u64 = slider_lookup(slider_type, square, board.occupancy()) & !board.occupancy_us() & allowed_squares & pin_ray_mask[square as usize];
            move_list.add_moves(slider_moves, square, MoveFlag::None);
        }
    }

    fn update_king_moves(board: &Board, move_list: &mut MoveList, pieces_checking: u64) {

        let attack_squares =  Self::get_enemy_attacks(board);
        let king_square = board.king_square(board.side_to_move());

        let mut valid_moves: u64 = (MOVEMENT_MASKS.king[king_square as usize]) & !board.occupancy_us() & !attack_squares;

        if GENERATOR_TYPE == GEN_TACTICS { 
            valid_moves &= board.occupancy_them();
            move_list.add_moves(valid_moves, king_square, MoveFlag::None);
            return;
        }
        
        move_list.add_moves(valid_moves, king_square, MoveFlag::None);

        if board.has_short_castle_rights(board.side_to_move()) && pieces_checking == 0{
            let clear_squares = if board.side_to_move().is_white() { 96 } else { 6917529027641081856 };

            if (clear_squares & attack_squares == 0) && (clear_squares & board.occupancy() == 0) {

                let move_to_square = if board.side_to_move().is_white() { Square::from(6) } else { Square::from(62) };
                move_list.add_moves(move_to_square.mask(), king_square, MoveFlag::CastleShort);

            }
        }

        if board.has_long_castle_rights(board.side_to_move()) && pieces_checking == 0{
            let not_attacked_squares: u64 = if board.side_to_move().is_white() { 12 } else { 864691128455135232 };
            let not_occupied_squares: u64 = if board.side_to_move().is_white() { 14 } else { 1008806316530991104 };

            if (not_attacked_squares & attack_squares == 0) && (not_occupied_squares & board.occupancy() == 0) {
                let move_to_index: u8 = if board.side_to_move().is_white() { 2 } else { 58 };

                move_list.add_moves(1 << move_to_index, king_square, MoveFlag::CastleLong);
            }

        }

    }

    
    
    fn get_pins(board: &Board, pin_ray_mask: &mut [u64; 64]) -> u64{
    
    
        let friendly_king_square = board.king_square(board.side_to_move());
        let friendly_pieces = board.occupancy_us();
        let enemy_pieces = board.occupancy_them();

        let enemy_orthogonal = board.orthogonal_bitboard_them();
        let enemy_diagonal = board.diagonal_bitboard_them();


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