use crate::chess::bitboard::Bitboard;
use crate::chess::board::Board;
use crate::chess::consts::PIECE_VALUES;
use crate::chess::move_ply::MovePly;
use crate::chess::types::color::Color;
use crate::chess::types::piece::BasePiece;
use crate::chess::types::piece::BasePiece::{Bishop, King, Knight, Pawn, Queen, Rook};
use crate::chess::types::square::Square;
use crate::general::bits;
use crate::precomputed::accessor::{
    MOVEMENT_MASKS, bishop_lookup, queen_lookup, rook_lookup, slider_lookup,
};

pub fn see(first_attacker_square: Square, square: Square, board: &Board) -> i16 {
    let first_victim = board.piece_at(square);
    if !first_victim.is_piece() {
        return 0;
    }

    let mut gains = [0i16; 32];
    let mut depth = 0;
    let mut occupancy = board.occupancy() & !first_attacker_square.mask();
    let mut occupied_by_piece = BasePiece::from(board.piece_at(first_attacker_square));
    let mut side_to_move = !board.side_to_move();

    gains[0] = PIECE_VALUES[first_victim as usize];

    loop {
        depth += 1;
        // The gain IF we recapture: we capture occupied_by_piece...
        gains[depth] = PIECE_VALUES[occupied_by_piece as usize] - gains[depth - 1];

        // ...but only if it's not a losing recapture (the key line)
        if gains[depth].max(-gains[depth - 1]) < 0 {
            break;
        }

        let Some((attacker_square, piece)) =
            least_valuable_attacker(square, side_to_move, occupancy, board)
        else {
            break;
        };

        occupancy &= !attacker_square.mask();
        occupied_by_piece = piece;
        side_to_move = !side_to_move;
    }

    // Resolve backwards — each side picks the best outcome
    while depth > 0 {
        depth -= 1;
        gains[depth] = -(-gains[depth]).max(gains[depth + 1]);
    }

    gains[0]
}

fn least_valuable_attacker(
    square: Square,
    side: Color,
    occupancy: u64,
    board: &Board,
) -> Option<(Square, BasePiece)> {
    // Check piece types LVA order: Pawn, Knight, Bishop, Rook, Queen, King
    let pawn_attacks =
        MOVEMENT_MASKS.pawn_attacks(!side, square) & board.bitboard(Pawn, side) & occupancy;
    if pawn_attacks != 0 {
        return Some((Square::from(bits::next(pawn_attacks)), Pawn));
    }

    let knight_attacks =
        MOVEMENT_MASKS.knight[square as usize] & board.bitboard(Knight, side) & occupancy;
    if knight_attacks != 0 {
        return Some((Square::from(bits::next(knight_attacks)), Knight));
    }

    let bishop_attacks =
        bishop_lookup(square, occupancy) & board.bitboard(Bishop, side) & occupancy;
    if bishop_attacks != 0 {
        return Some((Square::from(bits::next(bishop_attacks)), Bishop));
    }

    let rook_attacks = rook_lookup(square, occupancy) & board.bitboard(Rook, side) & occupancy;
    if rook_attacks != 0 {
        return Some((Square::from(bits::next(rook_attacks)), Rook));
    }

    let queen_attacks = queen_lookup(square, occupancy) & board.bitboard(Queen, side) & occupancy;
    if queen_attacks != 0 {
        return Some((Square::from(bits::next(queen_attacks)), Queen));
    }

    let king_attacks =
        MOVEMENT_MASKS.king[square as usize] & board.bitboard(King, side) & occupancy;
    if king_attacks != 0 {
        return Some((Square::from(bits::next(king_attacks)), King));
    }

    None
}

// pub fn see(mv: MovePly, board: &Board) -> i16 {
//     const PIECE_VALUES: [i16; 6] = [100, 320, 330, 500, 900, 20000];

//     let from = mv.from();
//     let to = mv.to();

//     let moving_piece = BasePiece::from(board.piece_at(from));
//     let captured_piece = board.piece_at(to);

//     if !captured_piece.is_piece() {
//         return 0;
//     }

//     let mut gain = [0i16; 32];
//     let mut depth = 0;

//     gain[0] = PIECE_VALUES[BasePiece::from(captured_piece) as usize];

//     let mut occupancy = board.occupancy();

//     let mut side = board.side_to_move();
//     let mut attackers = attackers_to(to, occupancy, board);

//     let mut from_set = from.mask();
//     let mut piece = moving_piece;

//     loop {
//         depth += 1;

//         gain[depth] = PIECE_VALUES[piece as usize] - gain[depth - 1];

//         if gain[depth].max(-gain[depth - 1]) < 0 {
//             break;
//         }

//         attackers &= occupancy;

//         occupancy ^= from_set;

//         attackers |= xray_attackers(to, occupancy, board);

//         side = !side;

//         let next = least_valuable_attacker(attackers, side, board);

//         if next.is_none() {
//             break;
//         }

//         let (sq, next_piece) = next.unwrap();

//         from_set = sq.mask();
//         piece = next_piece;
//     }

//     while depth > 0 {
//         depth -= 1;
//         gain[depth] = -gain[depth + 1].max(-gain[depth]);
//     }

//     gain[0]
// }

// pub fn see(first_attacker_square: Square, square: Square, board: &Board) -> i16 {
//     let first_victim = board.piece_at(square);

//     if !first_victim.is_piece() {
//         return 0;
//     }

//     let mut occupancy = board.occupancy() & !first_attacker_square.mask();

//     let mut gain: i16 = PIECE_VALUES[first_victim as usize];
//     let mut occupied_by_piece = BasePiece::from(board.piece_at(first_attacker_square));
//     let start_color = board.side_to_move();
//     let mut side_to_move = !board.side_to_move();

//     loop {
//         let attacker_square;
//         let piece;

//         let pawn_attacks = MOVEMENT_MASKS.pawn_attacks(!side_to_move, square)
//             & board.bitboard(Pawn, side_to_move)
//             & occupancy;
//         let knight_attacks = MOVEMENT_MASKS.knight[square as usize]
//             & board.bitboard(Knight, side_to_move)
//             & occupancy;
//         let bishop_attacks =
//             bishop_lookup(square, occupancy) & board.bitboard(Bishop, side_to_move) & occupancy;
//         let rook_attacks =
//             rook_lookup(square, occupancy) & board.bitboard(Rook, side_to_move) & occupancy;
//         let queen_attacks =
//             queen_lookup(square, occupancy) & board.bitboard(Queen, side_to_move) & occupancy;
//         let king_attacks =
//             MOVEMENT_MASKS.king[square as usize] & board.bitboard(King, side_to_move) & occupancy;

//         if pawn_attacks != 0 {
//             attacker_square = Square::from(bits::next(pawn_attacks));
//             piece = Pawn;
//         } else if knight_attacks != 0 {
//             attacker_square = Square::from(bits::next(knight_attacks));
//             piece = Knight;
//         } else if bishop_attacks != 0 {
//             attacker_square = Square::from(bits::next(bishop_attacks));
//             piece = Bishop;
//         } else if rook_attacks != 0 {
//             attacker_square = Square::from(bits::next(rook_attacks));
//             piece = Rook;
//         } else if queen_attacks != 0 {
//             attacker_square = Square::from(bits::next(queen_attacks));
//             piece = Queen;
//         } else if king_attacks != 0 {
//             attacker_square = Square::from(bits::next(king_attacks));
//             piece = King;
//         } else {
//             break;
//         }

//         if side_to_move == start_color {
//             gain += PIECE_VALUES[occupied_by_piece as usize];
//         } else {
//             gain -= PIECE_VALUES[occupied_by_piece as usize];
//         }

//         occupied_by_piece = piece;
//         occupancy &= !attacker_square.mask();
//         side_to_move = !side_to_move;
//     }

//     gain
// }

pub fn move_is_capture(board: &Board, played: &MovePly) -> bool {
    board.occupancy_them() & played.to().mask() != 0
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
