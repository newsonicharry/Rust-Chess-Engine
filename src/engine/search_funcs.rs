use std::cmp;

use crate::{
    chess::{
        board::Board,
        consts::PIECE_VALUES,
        move_generator::get_attacks,
        move_ply::MovePly,
        types::{
            color::Color::{self},
            piece::{
                BasePiece,
                BasePiece::{Bishop, King, Knight, Pawn, Queen, Rook},
            },
            square::Square,
        },
    },
    general::bits,
    precomputed::accessor::{
        MOVEMENT_MASKS, bishop_lookup, queen_lookup, rook_lookup, slider_lookup,
    },
};

fn is_valid_see(occupancy: u64, board: &Board, side: Color) -> bool {
    let king_square = board.king_square(side);
    for piece in [Bishop, Rook, Queen] {
        let mut enemy_bb = board.bitboard(piece, !side) & occupancy;

        while enemy_bb != 0 {
            let piece_square = Square::from(bits::next(enemy_bb));
            if get_attacks(!side, piece_square, piece, occupancy) & king_square.mask() != 0 {
                return false;
            }
            enemy_bb ^= piece_square.mask();
        }
    }

    true
}

pub fn see(first_attacker_square: Square, square: Square, board: &Board) -> i16 {
    let mut gain: [i16; 32] = [0; 32];
    let mut occupancy = board.occupancy();

    let mut side = !board.side_to_move();
    let mut victim = board.piece_at(first_attacker_square).into();
    occupancy &= !first_attacker_square.mask();
    gain[0] = PIECE_VALUES[board.piece_at(square) as usize];

    let mut i = 1;

    while let Some((from_square, next_attacker)) =
        least_valuable_attacker(square, side, occupancy, board)
    {
        occupancy ^= from_square.mask();

        gain[i] = PIECE_VALUES[victim as usize] - gain[i - 1];
        victim = next_attacker;

        i += 1;
        side = !side;

        if victim == BasePiece::King {
            break;
        }
    }

    for idx in (1..i).rev() {
        gain[idx - 1] = -cmp::max(-gain[idx - 1], gain[idx]);
    }

    gain[0]
}

fn least_valuable_attacker(
    square: Square,
    side: Color,
    occupancy: u64,
    board: &Board,
) -> Option<(Square, BasePiece)> {
    let mut pawn_attacks =
        MOVEMENT_MASKS.pawn_attacks(!side, square) & board.bitboard(Pawn, side) & occupancy;
    while pawn_attacks != 0 {
        let pawn_square = Square::from(bits::next(pawn_attacks));
        pawn_attacks ^= pawn_square.mask();
        if is_valid_see(occupancy ^ pawn_square.mask(), board, side) {
            return Some((pawn_square, Pawn));
        }
    }

    let mut knight_attacks =
        MOVEMENT_MASKS.knight[square as usize] & board.bitboard(Knight, side) & occupancy;
    while knight_attacks != 0 {
        let knight_square = Square::from(bits::next(knight_attacks));
        knight_attacks ^= knight_square.mask();
        if is_valid_see(occupancy ^ knight_square.mask(), board, side) {
            return Some((knight_square, Knight));
        }
    }

    let mut bishop_attacks =
        bishop_lookup(square, occupancy) & board.bitboard(Bishop, side) & occupancy;
    while bishop_attacks != 0 {
        let bishop_square = Square::from(bits::next(bishop_attacks));
        bishop_attacks ^= bishop_square.mask();

        if is_valid_see(occupancy ^ bishop_square.mask(), board, side) {
            return Some((bishop_square, Bishop));
        }
    }

    let mut rook_attacks = rook_lookup(square, occupancy) & board.bitboard(Rook, side) & occupancy;
    while rook_attacks != 0 {
        let rook_square = Square::from(bits::next(rook_attacks));
        rook_attacks ^= rook_square.mask();

        if is_valid_see(occupancy ^ rook_square.mask(), board, side) {
            return Some((rook_square, Rook));
        }
    }

    let mut queen_attacks =
        queen_lookup(square, occupancy) & board.bitboard(Queen, side) & occupancy;
    while queen_attacks != 0 {
        let queen_square = Square::from(bits::next(queen_attacks));
        queen_attacks ^= queen_square.mask();
        if is_valid_see(occupancy ^ queen_square.mask(), board, side) {
            return Some((queen_square, Queen));
        }
    }

    let king_attacks =
        MOVEMENT_MASKS.king[square as usize] & board.bitboard(King, side) & occupancy;
    if king_attacks != 0 {
        return Some((Square::from(bits::next(king_attacks)), King));
    }

    None
}

pub fn move_is_capture(board: &Board, played: &MovePly) -> bool {
    board.occupancy_them() & played.to().mask() != 0 || played.flag().is_en_passant_capture()
}

#[allow(dead_code)]
pub fn move_is_quiet(board: &Board, played: &MovePly) -> bool {
    if played.flag().is_promotion() {
        return false;
    }

    let capture = board.piece_at(played.to());
    if capture.is_piece() {
        return false;
    }

    let square = played.from();
    let piece = BasePiece::from(board.piece_at(square));
    let enemy_king_mask = board.king_square(!board.side_to_move()).mask();

    match piece {
        Pawn => {
            if MOVEMENT_MASKS.pawn_attacks(!board.side_to_move(), square) & enemy_king_mask != 0 {
                return false;
            }
        }

        Knight => {
            if MOVEMENT_MASKS.knight[square as usize] & enemy_king_mask != 0 {
                return false;
            }
        }

        Bishop => {
            if slider_lookup(Bishop, square, board.occupancy_them()) & enemy_king_mask != 0 {
                return false;
            }
        }

        Rook => {
            if slider_lookup(Rook, square, board.occupancy_them()) & enemy_king_mask != 0 {
                return false;
            }
        }

        Queen => {
            if slider_lookup(Queen, square, board.occupancy_them()) & enemy_king_mask != 0 {
                return false;
            }
        }

        _ => {}
    }

    true
}
