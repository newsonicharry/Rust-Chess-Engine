use std::u64;

use crate::chess::board::Board;
use crate::chess::consts::NUM_SQUARES;
use crate::chess::move_list::{MoveList, PieceMoves};
use crate::chess::types::color::Color;
use crate::chess::types::move_flag::MoveFlag;
use crate::chess::types::piece::BasePiece;
use crate::chess::types::piece::BasePiece::{Bishop, Knight, Pawn, Queen, Rook};
use crate::chess::types::square::Square;
use crate::general::bits;
use crate::precomputed::accessor::{
    IN_BETWEEN, MOVEMENT_MASKS, bishop_lookup, queen_lookup, rook_lookup, slider_lookup,
};

pub struct MoveGenerator<const GENERATOR_TYPE: bool> {}

pub const GEN_ALL: bool = false;
pub const GEN_TACTICS: bool = true;

const PIN_RAY_MASK_SIZE: usize = NUM_SQUARES + 1;

impl<const GENERATOR_TYPE: bool> MoveGenerator<GENERATOR_TYPE> {
    pub fn generate(board: *mut Board, move_iter: &mut impl FnMut(PieceMoves)) {
        let board: &mut Board = unsafe { &mut (*board) };

        board.update_occupancy();
        let (pieces_checking, allowed_squares) = Self::get_check_data(board);

        let mut pin_ray_mask: [u64; PIN_RAY_MASK_SIZE] = [u64::MAX; PIN_RAY_MASK_SIZE];
        pin_ray_mask[PIN_RAY_MASK_SIZE - 1] = 0;

        let pinned_pieces_mask = Self::get_pins(board, &mut pin_ray_mask);

        if pieces_checking != 0 {
            board.set_in_check(true);
        } else {
            board.set_in_check(false);
        }

        Self::update_pawn_moves(
            board,
            move_iter,
            allowed_squares,
            pinned_pieces_mask,
            &pin_ray_mask,
        );

        Self::update_knight_moves(board, move_iter, allowed_squares, pinned_pieces_mask);

        Self::update_king_moves(board, move_iter, pieces_checking);

        Self::update_slider_moves(Bishop, board, move_iter, allowed_squares, &pin_ray_mask);
        Self::update_slider_moves(Rook, board, move_iter, allowed_squares, &pin_ray_mask);
        Self::update_slider_moves(Queen, board, move_iter, allowed_squares, &pin_ray_mask);
    }

    fn pop_lsb(b: &mut u64) -> u32 {
        let index = b.trailing_zeros();
        *b &= *b - 1;
        index
    }

    fn get_attacks(
        side_to_move: Color,
        square: Square,
        piece: BasePiece,
        all_pieces_no_king: u64,
    ) -> u64 {
        match piece {
            BasePiece::Pawn => MOVEMENT_MASKS.pawn_attacks(!side_to_move, square),
            BasePiece::Knight => MOVEMENT_MASKS.knight[square as usize],
            BasePiece::Bishop => bishop_lookup(square, all_pieces_no_king),
            BasePiece::Rook => rook_lookup(square, all_pieces_no_king),
            BasePiece::Queen => queen_lookup(square, all_pieces_no_king),
            BasePiece::King => MOVEMENT_MASKS.king[square as usize],
        }
    }

    fn get_enemy_attacks(board: &Board) -> u64 {
        let mut attack_mask: u64 = 0;

        let all_pieces_no_king =
            board.occupancy() & !board.king_square(board.side_to_move()).mask();

        for piece in BasePiece::iterator() {
            let mut bitboard = board.bitboard_them(piece);
            while bitboard != 0 {
                let square = Self::pop_lsb(&mut bitboard).into();
                attack_mask |=
                    Self::get_attacks(board.side_to_move(), square, piece, all_pieces_no_king);
            }
        }

        attack_mask
    }

    fn get_check_data(board: &Board) -> (u64, u64) {
        let king_square = board.king_square(board.side_to_move());

        let enemy_orthogonal = board.orthogonal_bitboard_them();
        let enemy_diagonal = board.diagonal_bitboard_them();

        let knight_checks =
            MOVEMENT_MASKS.knight[king_square as usize] & board.bitboard_them(Knight);
        let pawn_checks = MOVEMENT_MASKS.pawn_attacks(board.side_to_move(), king_square)
            & board.bitboard_them(Pawn);

        let regular_check: u64 = knight_checks | pawn_checks;
        let orthogonal_check: u64 = rook_lookup(king_square, board.occupancy()) & enemy_orthogonal;
        let diagonal_check: u64 = bishop_lookup(king_square, board.occupancy()) & enemy_diagonal;

        let all_checks = orthogonal_check | diagonal_check | regular_check;
        let mut allowed_squares: u64 = 0;

        if bits::count(all_checks) == 1 {
            let index = bits::next(all_checks);
            allowed_squares =
                IN_BETWEEN.in_between[king_square as usize][index as usize] | (1 << index)
        }

        // there are no blockable squares (they are all valid so all on)
        if all_checks == 0 {
            allowed_squares = !allowed_squares;
        }

        (all_checks, allowed_squares)
    }

    // fn iter_single()

    fn iter_single(
        from: Square,
        to_mask: u64,
        flag: MoveFlag,
        move_iter: &mut impl FnMut(PieceMoves),
    ) {
        move_iter(PieceMoves::new(from, to_mask, flag))
    }

    fn iter_bulk_white(
        to_mask: u64,
        shift: u8,
        flag: MoveFlag,
        move_iter: &mut impl FnMut(PieceMoves),
    ) {
        move_iter(PieceMoves::new_bulk(to_mask, to_mask >> shift, flag))
    }

    fn iter_bulk_black(
        to_mask: u64,
        shift: u8,
        flag: MoveFlag,
        move_iter: &mut impl FnMut(PieceMoves),
    ) {
        move_iter(PieceMoves::new_bulk(to_mask, to_mask << shift, flag))
    }

    fn iter_promote_white(to_mask: u64, shift: u8, move_iter: &mut impl FnMut(PieceMoves)) {
        move_iter(PieceMoves::new_promote(to_mask, to_mask >> shift));
    }

    fn iter_promote_black(to_mask: u64, shift: u8, move_iter: &mut impl FnMut(PieceMoves)) {
        move_iter(PieceMoves::new_promote(to_mask, to_mask << shift));
    }

    fn update_pawn_moves(
        board: &Board,
        move_iter: &mut impl FnMut(PieceMoves),
        allowed_squares: u64,
        pinned_piece_mask: u64,
        pin_ray_mask: &[u64; PIN_RAY_MASK_SIZE],
    ) {
        const RIGHT: u64 = 0b1000000010000000100000001000000010000000100000001000000010000000;
        const LEFT: u64 = 0b0000000100000001000000010000000100000001000000010000000100000001;

        const WHITE_DOUBLE_JUMP: u64 =
            0b0000000000000000000000000000000000000000000000001111111100000000;
        const WHITE_CAN_DOUBLE_JUMP: u64 =
            0b0000000000000000000000000000000000000000111111110000000000000000;
        const WHITE_PROMOTE: u64 =
            0b1111111100000000000000000000000000000000000000000000000000000000;

        const BLACK_DOUBLE_JUMP: u64 =
            0b0000000011111111000000000000000000000000000000000000000000000000;
        const BLACK_CAN_DOUBLE_JUMP: u64 =
            0b0000000000000000111111110000000000000000000000000000000000000000;

        const BLACK_PROMOTE: u64 =
            0b0000000000000000000000000000000000000000000000000000000011111111;

        let pawn_bitboard = board.bitboard(Pawn, board.side_to_move()) & !pinned_piece_mask;
        let mut pinned_pawn_bitboard =
            board.bitboard(Pawn, board.side_to_move()) & pinned_piece_mask;

        let middle_pawns = pawn_bitboard & !LEFT & !RIGHT;

        let mut single_push;
        let mut double_push;

        let mut single_push_promotions;
        let mut left_attack_promotions;
        let mut right_attack_promotions;

        let mut left_pawn_attacks;
        let mut right_pawn_attacks;

        if board.side_to_move().is_white() {
            right_pawn_attacks = ((pawn_bitboard & LEFT) | middle_pawns) << 9;
            left_pawn_attacks = ((pawn_bitboard & RIGHT) | middle_pawns) << 7;

            single_push = pawn_bitboard << 8;
            double_push = (pawn_bitboard & WHITE_DOUBLE_JUMP) << 16;

            while pinned_pawn_bitboard != 0 {
                let square = bits::next(pinned_pawn_bitboard);
                let square_mask = 1 << square;
                let pin_mask = pin_ray_mask[square as usize];

                single_push |= (square_mask << 8) & pin_mask;
                double_push |= ((square_mask & WHITE_DOUBLE_JUMP) << 16) & pin_mask;

                left_pawn_attacks |= ((square_mask & !LEFT) << 7) & pin_mask;
                right_pawn_attacks |= ((square_mask & !RIGHT) << 9) & pin_mask;

                pinned_pawn_bitboard &= pinned_pawn_bitboard - 1;
            }

            double_push &= !((WHITE_CAN_DOUBLE_JUMP & board.occupancy()) << 8);

            single_push_promotions = single_push & WHITE_PROMOTE;
            left_attack_promotions = left_pawn_attacks & WHITE_PROMOTE;
            right_attack_promotions = right_pawn_attacks & WHITE_PROMOTE;

            single_push &= !WHITE_PROMOTE;
            right_pawn_attacks &= !WHITE_PROMOTE;
            left_pawn_attacks &= !WHITE_PROMOTE;
        } else {
            right_pawn_attacks = ((pawn_bitboard & LEFT) | middle_pawns) >> 7;
            left_pawn_attacks = ((pawn_bitboard & RIGHT) | middle_pawns) >> 9;

            single_push = pawn_bitboard >> 8;

            double_push = (pawn_bitboard & BLACK_DOUBLE_JUMP) >> 16;

            while pinned_pawn_bitboard != 0 {
                let square = bits::next(pinned_pawn_bitboard);
                let square_mask = 1 << square;
                let pin_mask = pin_ray_mask[square as usize];

                single_push |= (square_mask >> 8) & pin_mask;
                double_push |= ((square_mask & BLACK_DOUBLE_JUMP) >> 16) & pin_mask;

                left_pawn_attacks |= ((square_mask & !LEFT) >> 9) & pin_mask;
                right_pawn_attacks |= ((square_mask & !RIGHT) >> 7) & pin_mask;

                pinned_pawn_bitboard &= pinned_pawn_bitboard - 1;
            }

            double_push &= !((BLACK_CAN_DOUBLE_JUMP & board.occupancy()) >> 8);

            single_push_promotions = single_push & BLACK_PROMOTE;
            left_attack_promotions = left_pawn_attacks & BLACK_PROMOTE;
            right_attack_promotions = right_pawn_attacks & BLACK_PROMOTE;

            single_push &= !BLACK_PROMOTE;
            right_pawn_attacks &= !BLACK_PROMOTE;
            left_pawn_attacks &= !BLACK_PROMOTE;
        }

        let raw_left_attacks = left_pawn_attacks;
        let raw_right_attacks = right_pawn_attacks;

        left_pawn_attacks &= board.occupancy_them() & allowed_squares;
        right_pawn_attacks &= board.occupancy_them() & allowed_squares;

        left_attack_promotions &= board.occupancy_them() & allowed_squares;
        right_attack_promotions &= board.occupancy_them() & allowed_squares;

        single_push_promotions &= !board.occupancy() & allowed_squares;
        single_push &= !board.occupancy() & allowed_squares;
        double_push &= !board.occupancy() & allowed_squares;

        if board.side_to_move().is_white() {
            if GENERATOR_TYPE == GEN_ALL {
                Self::iter_bulk_white(double_push, 16, MoveFlag::DoubleJump, move_iter);
                Self::iter_bulk_white(single_push, 8, MoveFlag::None, move_iter);
            }

            Self::iter_bulk_white(left_pawn_attacks, 7, MoveFlag::None, move_iter);
            Self::iter_bulk_white(right_pawn_attacks, 9, MoveFlag::None, move_iter);

            Self::iter_promote_white(single_push_promotions, 8, move_iter);
            Self::iter_promote_white(left_attack_promotions, 7, move_iter);
            Self::iter_promote_white(right_attack_promotions, 9, move_iter);
        } else {
            if GENERATOR_TYPE == GEN_ALL {
                Self::iter_bulk_black(double_push, 16, MoveFlag::DoubleJump, move_iter);
                Self::iter_bulk_black(single_push, 8, MoveFlag::None, move_iter);
            }

            Self::iter_bulk_black(left_pawn_attacks, 9, MoveFlag::None, move_iter);
            Self::iter_bulk_black(right_pawn_attacks, 7, MoveFlag::None, move_iter);

            Self::iter_promote_black(single_push_promotions, 8, move_iter);
            Self::iter_promote_black(left_attack_promotions, 9, move_iter);
            Self::iter_promote_black(right_attack_promotions, 7, move_iter);
        }

        let king_square = board.king_square(board.side_to_move());

        let calculate_discovered_enpassant_attack =
            |pawn_mask: u64, en_passant_attack_mask: u64, enemy_pawn_mask: u64| -> u64 {
                if pawn_mask == 0 {
                    return 0;
                }

                let new_blockers =
                    board.occupancy() & (!enemy_pawn_mask) & (!pawn_mask) | en_passant_attack_mask;

                let enemy_orthogonal = board.orthogonal_bitboard_them();
                let enemy_diagonal = board.diagonal_bitboard_them();

                let no_orthogonal_attack =
                    rook_lookup(king_square, new_blockers) & enemy_orthogonal == 0;
                let no_diagonal_attack =
                    bishop_lookup(king_square, new_blockers) & enemy_diagonal == 0;

                if no_orthogonal_attack && no_diagonal_attack {
                    return pawn_mask;
                }

                0
            };

        if let Some(en_passant_file) = board.en_passant_file() {
            if board.side_to_move().is_white() {
                let en_passant_attack_square = en_passant_file as u8 + 40;
                let en_passant_attack_mask = 1 << en_passant_attack_square;

                let right_pawn_attack = raw_left_attacks & en_passant_attack_mask;
                let left_pawn_attack = raw_right_attacks & en_passant_attack_mask;

                let mut right_pawn_position = right_pawn_attack >> 7;
                let mut left_pawn_position = left_pawn_attack >> 9;

                right_pawn_position &= pin_ray_mask[right_pawn_position.trailing_zeros() as usize];
                left_pawn_position &= pin_ray_mask[left_pawn_position.trailing_zeros() as usize];

                let enemy_pawn_mask = 1 << (en_passant_file as u8 + 32);
                right_pawn_position = calculate_discovered_enpassant_attack(
                    right_pawn_position,
                    en_passant_attack_mask,
                    enemy_pawn_mask,
                );

                left_pawn_position = calculate_discovered_enpassant_attack(
                    left_pawn_position,
                    en_passant_attack_mask,
                    enemy_pawn_mask,
                );

                if right_pawn_position != 0 {
                    Self::iter_single(
                        Square::from(bits::next(right_pawn_position)),
                        en_passant_attack_mask,
                        MoveFlag::EnPassantCapture,
                        move_iter,
                    );
                }

                if left_pawn_position != 0 {
                    Self::iter_single(
                        Square::from(bits::next(left_pawn_position)),
                        en_passant_attack_mask,
                        MoveFlag::EnPassantCapture,
                        move_iter,
                    );
                }

                // Self::iter_single(
                //     Square::from(en_passant_attack_square),
                //     right_pawn_position | left_pawn_position,
                //     MoveFlag::EnPassantCapture,
                //     move_iter,
                // );

                // move_list.add_enpassant_moves(
                //     Square::from(en_passant_attack_square),
                //     right_pawn_position | left_pawn_position,
                // );
            } else {
                let en_passant_attack_square = en_passant_file as u8 + 16;
                let en_passant_attack_mask = 1 << en_passant_attack_square;

                let mut right_pawn_attack = raw_left_attacks & en_passant_attack_mask;
                let mut left_pawn_attack = raw_right_attacks & en_passant_attack_mask;

                right_pawn_attack &= pin_ray_mask[right_pawn_attack.trailing_zeros() as usize];
                left_pawn_attack &= pin_ray_mask[left_pawn_attack.trailing_zeros() as usize];

                let mut right_pawn_position = right_pawn_attack << 9;
                let mut left_pawn_position = left_pawn_attack << 7;

                let enemy_pawn_mask = 1 << (en_passant_file as u8 + 24);
                right_pawn_position = calculate_discovered_enpassant_attack(
                    right_pawn_position,
                    en_passant_attack_mask,
                    enemy_pawn_mask,
                );

                left_pawn_position = calculate_discovered_enpassant_attack(
                    left_pawn_position,
                    en_passant_attack_mask,
                    enemy_pawn_mask,
                );

                if right_pawn_position != 0 {
                    Self::iter_single(
                        Square::from(bits::next(right_pawn_position)),
                        en_passant_attack_mask,
                        MoveFlag::EnPassantCapture,
                        move_iter,
                    );
                }

                if left_pawn_position != 0 {
                    Self::iter_single(
                        Square::from(bits::next(left_pawn_position)),
                        en_passant_attack_mask,
                        MoveFlag::EnPassantCapture,
                        move_iter,
                    );
                }
                // Self::iter_single(
                //     Square::from(en_passant_attack_square),
                //     right_pawn_position | left_pawn_position,
                //     MoveFlag::EnPassantCapture,
                //     move_iter,
                // );

                // move_list.add_enpassant_moves(
                //     Square::from(en_passant_attack_square),
                //     right_pawn_position | left_pawn_position,
                // );
            }
        }
    }

    fn update_knight_moves(
        board: &Board,
        // move_list: &mut MoveList,
        move_iter: &mut impl FnMut(PieceMoves),
        mut allowed_squares: u64,
        pinned_pieces_mask: u64,
    ) {
        if GENERATOR_TYPE == GEN_TACTICS {
            let enemy_king = board.king_square(!board.side_to_move());
            let checking_squares = MOVEMENT_MASKS.knight[enemy_king as usize];

            allowed_squares =
                (allowed_squares & board.occupancy_them()) | (allowed_squares & checking_squares);
        }

        let mut knight_bitboard = board.bitboard(Knight, board.side_to_move());
        while knight_bitboard != 0 {
            let square: Square = Self::pop_lsb(&mut knight_bitboard).into();

            if pinned_pieces_mask & square.mask() != 0 {
                continue;
            }

            let knight_moves: u64 =
                (MOVEMENT_MASKS.knight[square as usize]) & !board.occupancy_us() & allowed_squares;

            Self::iter_single(square, knight_moves, MoveFlag::None, move_iter);
        }
    }

    fn update_slider_moves(
        slider_type: BasePiece,
        board: &Board,
        move_iter: &mut impl FnMut(PieceMoves),
        // move_list: &mut MoveList,
        mut allowed_squares: u64,
        pin_ray_mask: &[u64; PIN_RAY_MASK_SIZE],
    ) {
        if GENERATOR_TYPE == GEN_TACTICS {
            let enemy_king = board.king_square(!board.side_to_move());
            let checking_squares = slider_lookup(slider_type, enemy_king, board.occupancy());

            allowed_squares =
                (allowed_squares & board.occupancy_them()) | (allowed_squares & checking_squares);
        }

        let mut slider_bitboard = board.bitboard(slider_type, board.side_to_move());
        while slider_bitboard != 0 {
            let square: Square = Self::pop_lsb(&mut slider_bitboard).into();

            // for &square in board.piece_list_us(slider_type) {
            let slider_moves: u64 = slider_lookup(slider_type, square, board.occupancy())
                & !board.occupancy_us()
                & allowed_squares
                & pin_ray_mask[square as usize];
            // move_list.add_moves(slider_moves, square, MoveFlag::None);

            Self::iter_single(square, slider_moves, MoveFlag::None, move_iter);
        }
    }

    fn update_king_moves(
        board: &Board,
        move_iter: &mut impl FnMut(PieceMoves),
        pieces_checking: u64,
    ) {
        let attack_squares = Self::get_enemy_attacks(board);
        let king_square = board.king_square(board.side_to_move());

        let mut valid_moves: u64 =
            (MOVEMENT_MASKS.king[king_square as usize]) & !board.occupancy_us() & !attack_squares;

        if GENERATOR_TYPE == GEN_TACTICS {
            valid_moves &= board.occupancy_them();
            // move_list.add_moves(valid_moves, king_square, MoveFlag::None);
            Self::iter_single(king_square, valid_moves, MoveFlag::None, move_iter);
            return;
        }

        // move_list.add_moves(valid_moves, king_square, MoveFlag::None);
        Self::iter_single(king_square, valid_moves, MoveFlag::None, move_iter);

        if board.has_short_castle_rights(board.side_to_move()) && pieces_checking == 0 {
            let clear_squares = if board.side_to_move().is_white() {
                96
            } else {
                6917529027641081856
            };

            if (clear_squares & attack_squares == 0) && (clear_squares & board.occupancy() == 0) {
                let move_to_square = if board.side_to_move().is_white() {
                    Square::G1
                } else {
                    Square::G8
                };
                // move_list.add_moves(move_to_square.mask(), king_square, MoveFlag::CastleShort);

                Self::iter_single(
                    king_square,
                    move_to_square.mask(),
                    MoveFlag::CastleShort,
                    move_iter,
                );
            }
        }

        if board.has_long_castle_rights(board.side_to_move()) && pieces_checking == 0 {
            let not_attacked_squares: u64 = if board.side_to_move().is_white() {
                12
            } else {
                864691128455135232
            };
            let not_occupied_squares: u64 = if board.side_to_move().is_white() {
                14
            } else {
                1008806316530991104
            };

            if (not_attacked_squares & attack_squares == 0)
                && (not_occupied_squares & board.occupancy() == 0)
            {
                let move_to_square = if board.side_to_move().is_white() {
                    Square::C1
                } else {
                    Square::C8
                };

                Self::iter_single(
                    king_square,
                    move_to_square.mask(),
                    MoveFlag::CastleLong,
                    move_iter,
                );
                // move_list.add_moves(move_to_square.mask(), king_square, MoveFlag::CastleLong);
            }
        }
    }

    fn get_pins(board: &Board, pin_ray_mask: &mut [u64; PIN_RAY_MASK_SIZE]) -> u64 {
        let friendly_king_square = board.king_square(board.side_to_move());
        let friendly_pieces = board.occupancy_us();
        let enemy_pieces = board.occupancy_them();

        let enemy_orthogonal = board.orthogonal_bitboard_them();
        let enemy_diagonal = board.diagonal_bitboard_them();

        let possible_orthogonally_pinned: u64 =
            MOVEMENT_MASKS.rook[friendly_king_square as usize] & enemy_orthogonal;
        let possible_diagonally_pinned: u64 =
            MOVEMENT_MASKS.bishop[friendly_king_square as usize] & enemy_diagonal;

        let mut possible_pinners = possible_orthogonally_pinned | possible_diagonally_pinned;

        let mut pinned_pieces_mask: u64 = 0;

        while possible_pinners != 0 {
            let possible_pinner = bits::next(possible_pinners);
            let ray =
                IN_BETWEEN.in_between[friendly_king_square as usize][possible_pinner as usize];

            // opponents between the king and pinner
            if (ray & enemy_pieces) != 0 {
                possible_pinners = bits::pop(possible_pinners);
                continue;
            }

            let friendly_pieces_between = ray & friendly_pieces;

            if bits::count(friendly_pieces_between) == 1 {
                pinned_pieces_mask |= friendly_pieces_between;
                let friendly_piece_index = bits::next(friendly_pieces_between) as usize;
                pin_ray_mask[friendly_piece_index] = IN_BETWEEN.in_between
                    [friendly_king_square as usize][possible_pinner as usize]
                    | (1 << possible_pinner);
            }

            possible_pinners = bits::pop(possible_pinners);
        }

        pinned_pieces_mask
    }
}
