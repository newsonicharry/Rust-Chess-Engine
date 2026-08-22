use crate::chess::board::Board;
use crate::chess::consts::NUM_SQUARES;
use crate::chess::move_list::PieceMoves;
use crate::chess::types::color::Color;
use crate::chess::types::move_flag::MoveFlag;
use crate::chess::types::piece::BasePiece;
use crate::chess::types::piece::BasePiece::{Bishop, Knight, Pawn, Queen, Rook};
use crate::chess::types::square::Square;
use crate::general::bits;
use crate::precomputed::accessor::{
    IN_BETWEEN, KING_ATTACK_MASKS, MOVEMENT_MASKS, bishop_lookup, queen_lookup, rook_lookup,
    slider_lookup,
};
use crate::precomputed::generators::king_attack_masks::KingAttackLookupData;

pub struct MoveGenerator<const GENERATOR_TYPE: bool> {}

pub const IS_LEAF: bool = true;
pub const NOT_LEAF: bool = false;

pub const GEN_ALL: bool = false;
pub const GEN_TACTICS: bool = true;

pub const WHITE: bool = true;
pub const BLACK: bool = false;

const PIN_RAY_MASK_SIZE: usize = NUM_SQUARES + 1;

// bitboard of squares that cannot be attacked inorder for castling to be possible
// (since the king cannot castle through check)
pub const WHITE_SHORT_NOT_ATTACKED_SQUARES: u64 = 96;
pub const BLACK_SHORT_NOT_ATTACKED_SQUARES: u64 = 0x6000000000000000;

pub const WHITE_LONG_NOT_ATTACKED_SQUARES: u64 = 12;
pub const BLACK_LONG_NOT_ATTACKED_SQUARES: u64 = 0xC00000000000000;

pub const WHITE_LONG_NOT_OCCUPIED_SQUARES: u64 = 14;
pub const BLACK_LONG_NOT_OCCUPIED_SQUARES: u64 = 0xE00000000000000;

impl<const GENERATOR_TYPE: bool> MoveGenerator<GENERATOR_TYPE> {
    pub fn generate<const IS_LEAF: bool>(
        board: *mut Board,
        move_iter: &mut impl FnMut(PieceMoves),
    ) -> u64 {
        let board: &mut Board = unsafe { &mut (*board) };

        match board.side_to_move() {
            Color::White => Self::const_generate::<WHITE, IS_LEAF>(board, move_iter),
            Color::Black => Self::const_generate::<BLACK, IS_LEAF>(board, move_iter),
        }
    }

    #[inline(always)]
    pub fn const_generate<const COLOR: bool, const IS_LEAF: bool>(
        board: *mut Board,
        move_iter: &mut impl FnMut(PieceMoves),
    ) -> u64 {
        let board: &mut Board = unsafe { &mut (*board) };

        let (pieces_checking, allowed_squares) = Self::get_check_data::<COLOR>(board);

        let mut pin_ray_mask: [u64; PIN_RAY_MASK_SIZE] = [u64::MAX; PIN_RAY_MASK_SIZE];
        pin_ray_mask[PIN_RAY_MASK_SIZE - 1] = 0;

        let pinned_pieces_mask = Self::get_pins::<COLOR>(board, &mut pin_ray_mask);

        if pieces_checking != 0 {
            board.set_in_check(true);
        } else {
            board.set_in_check(false);
        }

        let mut count = 0;

        count += Self::update_pawn_moves::<COLOR, IS_LEAF>(
            board,
            move_iter,
            allowed_squares,
            pinned_pieces_mask,
            &pin_ray_mask,
        );

        count += Self::update_knight_moves::<COLOR, IS_LEAF>(
            board,
            move_iter,
            allowed_squares,
            pinned_pieces_mask,
        );

        count += Self::update_king_moves::<COLOR, IS_LEAF>(board, move_iter, pieces_checking);

        for piece in [Bishop, Rook, Queen] {
            count += Self::update_slider_moves::<COLOR, IS_LEAF>(
                piece,
                board,
                move_iter,
                allowed_squares,
                &pin_ray_mask,
            );
        }

        return count as u64;
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

    #[inline(always)]
    fn get_enemy_attacks<const COLOR: bool>(
        board: &Board,
        potential_short_castle: bool,
        potential_long_castle: bool,
    ) -> u64 {
        macro_rules! attack_mask_creation {
            ($bitboard: ident, $piece: expr, $attack_mask: ident, $board: ident, $all_pieces_no_king: ident) => {{
                while $bitboard != 0 {
                    let square = Self::pop_lsb(&mut $bitboard).into();
                    $attack_mask |= Self::get_attacks(
                        board.side_to_move(),
                        square,
                        $piece,
                        $all_pieces_no_king,
                    );
                }
            }};
        }

        let mut attack_mask: u64 = 0;
        let king_square = board.king_square_const::<COLOR>();
        let king_mask_occupancy =
            MOVEMENT_MASKS.king[king_square as usize] & board.occupancy_us_const::<COLOR>();

        let occupancy = board.occupancy() & !board.king_square_const::<COLOR>().mask();

        let lookup_data = KingAttackLookupData {
            king_square,
            king_mask_occupancy,
            side_to_move: board.side_to_move(),
            potential_short_castle,
            potential_long_castle,
        };

        let them: Color = (!COLOR).into();

        let mut pawns =
            board.bitboard(BasePiece::Pawn, them) & KING_ATTACK_MASKS.pawn_lookup(&lookup_data);

        let mut knights =
            board.bitboard(BasePiece::Knight, them) & KING_ATTACK_MASKS.knight_lookup(&lookup_data);

        let mut bishops =
            board.bitboard(BasePiece::Bishop, them) & KING_ATTACK_MASKS.bishop_lookup(&lookup_data);

        let mut rooks =
            board.bitboard(BasePiece::Rook, them) & KING_ATTACK_MASKS.rook_lookup(&lookup_data);

        let mut queens =
            board.bitboard(BasePiece::Queen, them) & KING_ATTACK_MASKS.queen_lookup(&lookup_data);

        let mut kings =
            board.bitboard(BasePiece::King, them) & KING_ATTACK_MASKS.king_lookup(&lookup_data);

        attack_mask_creation!(pawns, BasePiece::Pawn, attack_mask, board, occupancy);
        attack_mask_creation!(knights, BasePiece::Knight, attack_mask, board, occupancy);
        attack_mask_creation!(bishops, BasePiece::Bishop, attack_mask, board, occupancy);
        attack_mask_creation!(rooks, BasePiece::Rook, attack_mask, board, occupancy);
        attack_mask_creation!(queens, BasePiece::Queen, attack_mask, board, occupancy);
        attack_mask_creation!(kings, BasePiece::King, attack_mask, board, occupancy);

        attack_mask
    }

    #[inline(always)]
    fn get_check_data<const COLOR: bool>(board: &Board) -> (u64, u64) {
        let king_square = board.king_square_const::<COLOR>();

        let enemy_orthogonal = board.orthogonal_bitboard_them::<COLOR>();
        let enemy_diagonal = board.diagonal_bitboard_them::<COLOR>();

        let knight_checks =
            MOVEMENT_MASKS.knight[king_square as usize] & board.bitboard(Knight, (!COLOR).into());
        let pawn_checks = MOVEMENT_MASKS.pawn_attacks_const::<COLOR>(king_square)
            & board.bitboard(Pawn, (!COLOR).into());

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

    #[inline(always)]
    fn update_pawn_moves<const COLOR: bool, const IS_LEAF: bool>(
        board: &Board,
        move_iter: &mut impl FnMut(PieceMoves),
        allowed_squares: u64,
        pinned_piece_mask: u64,
        pin_ray_mask: &[u64; PIN_RAY_MASK_SIZE],
    ) -> u32 {
        const RIGHT: u64 = 0x8080808080808080;
        const LEFT: u64 = 0x101010101010101;

        const WHITE_DOUBLE_JUMP: u64 = 0xFF00;
        const WHITE_CAN_DOUBLE_JUMP: u64 = 0xFF0000;
        const WHITE_PROMOTE: u64 = 0xFF00000000000000;

        const BLACK_DOUBLE_JUMP: u64 = 0xFF000000000000;
        const BLACK_CAN_DOUBLE_JUMP: u64 = 0xFF0000000000;

        const BLACK_PROMOTE: u64 = 0xFF;

        let pawn_bitboard = board.bitboard_const::<COLOR>(Pawn) & !pinned_piece_mask;
        let mut pinned_pawn_bitboard = board.bitboard_const::<COLOR>(Pawn) & pinned_piece_mask;

        let middle_pawns = pawn_bitboard & !LEFT & !RIGHT;

        let mut single_push;
        let mut double_push;

        let mut single_push_promotions;
        let mut left_attack_promotions;
        let mut right_attack_promotions;

        let mut left_pawn_attacks;
        let mut right_pawn_attacks;

        if COLOR == WHITE {
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

        left_pawn_attacks &= board.occupancy_them_const::<COLOR>() & allowed_squares;
        right_pawn_attacks &= board.occupancy_them_const::<COLOR>() & allowed_squares;

        left_attack_promotions &= board.occupancy_them_const::<COLOR>() & allowed_squares;
        right_attack_promotions &= board.occupancy_them_const::<COLOR>() & allowed_squares;

        single_push_promotions &= !board.occupancy() & allowed_squares;
        single_push &= !board.occupancy() & allowed_squares;
        double_push &= !board.occupancy() & allowed_squares;

        let mut count: u32 = 0;

        if IS_LEAF {
            count += double_push.count_ones() + single_push.count_ones();
            count += left_pawn_attacks.count_ones() + right_pawn_attacks.count_ones();
            count += single_push_promotions.count_ones() * 4
                + left_attack_promotions.count_ones() * 4
                + right_attack_promotions.count_ones() * 4;
        } else if COLOR == WHITE {
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

        let king_square = board.king_square_const::<COLOR>();

        let calculate_discovered_enpassant_attack =
            |pawn_mask: u64, en_passant_attack_mask: u64, enemy_pawn_mask: u64| -> u64 {
                if pawn_mask == 0 {
                    return 0;
                }

                let new_blockers =
                    board.occupancy() & (!enemy_pawn_mask) & (!pawn_mask) | en_passant_attack_mask;

                let enemy_orthogonal = board.orthogonal_bitboard_them::<COLOR>();
                let enemy_diagonal = board.diagonal_bitboard_them::<COLOR>();

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
            if COLOR == WHITE {
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

                if IS_LEAF {
                    count += right_pawn_position.count_ones();
                    count += left_pawn_position.count_ones();
                    return count;
                }

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

                if IS_LEAF {
                    count += right_pawn_position.count_ones();
                    count += left_pawn_position.count_ones();
                    return count;
                }

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
            }
        }

        return count;
    }

    #[inline(always)]
    fn update_knight_moves<const COLOR: bool, const IS_LEAF: bool>(
        board: &Board,
        move_iter: &mut impl FnMut(PieceMoves),
        mut allowed_squares: u64,
        pinned_pieces_mask: u64,
    ) -> u32 {
        if GENERATOR_TYPE == GEN_TACTICS {
            let enemy_king = board.king_square_const::<COLOR>();
            let checking_squares = MOVEMENT_MASKS.knight[enemy_king as usize];

            allowed_squares = (allowed_squares & board.occupancy_them_const::<COLOR>())
                | (allowed_squares & checking_squares);
        }

        let mut count = 0;
        let mut knight_bitboard = board.bitboard_const::<COLOR>(Knight);

        while knight_bitboard != 0 {
            let square: Square = Self::pop_lsb(&mut knight_bitboard).into();

            if pinned_pieces_mask & square.mask() != 0 {
                continue;
            }

            let knight_moves: u64 = (MOVEMENT_MASKS.knight[square as usize])
                & !board.occupancy_us_const::<COLOR>()
                & allowed_squares;

            if IS_LEAF {
                count += knight_moves.count_ones();
            } else {
                Self::iter_single(square, knight_moves, MoveFlag::None, move_iter);
            }
        }

        return count;
    }

    #[inline(always)]
    fn update_slider_moves<const COLOR: bool, const IS_LEAF: bool>(
        slider_type: BasePiece,
        board: &Board,
        move_iter: &mut impl FnMut(PieceMoves),
        mut allowed_squares: u64,
        pin_ray_mask: &[u64; PIN_RAY_MASK_SIZE],
    ) -> u32 {
        if GENERATOR_TYPE == GEN_TACTICS {
            let enemy_king = board.king_square_const::<COLOR>();
            let checking_squares = slider_lookup(slider_type, enemy_king, board.occupancy());

            allowed_squares = (allowed_squares & board.occupancy_them_const::<COLOR>())
                | (allowed_squares & checking_squares);
        }

        let mut count = 0;
        let mut slider_bitboard = board.bitboard_const::<COLOR>(slider_type);
        while slider_bitboard != 0 {
            let square: Square = Self::pop_lsb(&mut slider_bitboard).into();

            let slider_moves: u64 = slider_lookup(slider_type, square, board.occupancy())
                & !board.occupancy_us_const::<COLOR>()
                & allowed_squares
                & pin_ray_mask[square as usize];

            if IS_LEAF {
                count += slider_moves.count_ones();
            } else {
                Self::iter_single(square, slider_moves, MoveFlag::None, move_iter);
            }
        }

        return count;
    }

    #[inline(always)]
    fn update_king_moves<const COLOR: bool, const IS_LEAF: bool>(
        board: &Board,
        move_iter: &mut impl FnMut(PieceMoves),
        pieces_checking: u64,
    ) -> u32 {
        let short_castle_not_occupied = match COLOR {
            WHITE => WHITE_SHORT_NOT_ATTACKED_SQUARES,
            BLACK => BLACK_SHORT_NOT_ATTACKED_SQUARES,
        };

        let long_castle_not_occupied = match COLOR {
            WHITE => WHITE_LONG_NOT_OCCUPIED_SQUARES,
            BLACK => BLACK_LONG_NOT_OCCUPIED_SQUARES,
        };

        let potental_short_castle = board.has_short_castle_rights_const::<COLOR>()
            && pieces_checking == 0
            && short_castle_not_occupied & board.occupancy() == 0;

        let potental_long_castle = board.has_long_castle_rights_const::<COLOR>()
            && pieces_checking == 0
            && long_castle_not_occupied & board.occupancy() == 0;

        let king_square = board.king_square_const::<COLOR>();

        let mut valid_moves: u64 =
            (MOVEMENT_MASKS.king[king_square as usize]) & !board.occupancy_us_const::<COLOR>();

        let mut attack_squares = 0;

        if valid_moves != 0 {
            attack_squares = Self::get_enemy_attacks::<COLOR>(
                board,
                potental_short_castle,
                potental_long_castle,
            );
            valid_moves &= !attack_squares
        }

        if GENERATOR_TYPE == GEN_TACTICS {
            valid_moves &= board.occupancy_them_const::<COLOR>();
            Self::iter_single(king_square, valid_moves, MoveFlag::None, move_iter);
            return 0;
        }

        let mut count = 0;
        if IS_LEAF {
            count += valid_moves.count_ones();
        } else {
            Self::iter_single(king_square, valid_moves, MoveFlag::None, move_iter);
        }

        if potental_short_castle && (short_castle_not_occupied & attack_squares == 0) {
            let move_to_square = match COLOR {
                WHITE => Square::G1,
                BLACK => Square::G8,
            };

            if IS_LEAF {
                count += 1;
            } else {
                Self::iter_single(
                    king_square,
                    move_to_square.mask(),
                    MoveFlag::CastleShort,
                    move_iter,
                );
            }
        }

        let not_attacked_squares = match COLOR {
            WHITE => WHITE_LONG_NOT_ATTACKED_SQUARES,
            BLACK => BLACK_LONG_NOT_ATTACKED_SQUARES,
        };
        if potental_long_castle && not_attacked_squares & attack_squares == 0 {
            let move_to_square = match COLOR {
                WHITE => Square::C1,
                BLACK => Square::C8,
            };

            if IS_LEAF {
                count += 1;
            } else {
                Self::iter_single(
                    king_square,
                    move_to_square.mask(),
                    MoveFlag::CastleLong,
                    move_iter,
                );
            }
        }

        return count;
    }

    #[inline(always)]
    fn get_pins<const COLOR: bool>(
        board: &Board,
        pin_ray_mask: &mut [u64; PIN_RAY_MASK_SIZE],
    ) -> u64 {
        let friendly_king_square = board.king_square_const::<COLOR>();
        let friendly_pieces = board.occupancy_us_const::<COLOR>();
        let enemy_pieces = board.occupancy_them_const::<COLOR>();

        let enemy_orthogonal = board.orthogonal_bitboard_them::<COLOR>();
        let enemy_diagonal = board.diagonal_bitboard_them::<COLOR>();

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
