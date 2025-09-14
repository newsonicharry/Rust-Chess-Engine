use crate::chess::board::Board;
use crate::chess::consts::PIECE_VALUES;
use crate::chess::precomputed::accessor::{bishop_lookup, queen_lookup, rook_lookup, MOVEMENT_MASKS};
use crate::chess::types::piece::BasePiece::{Bishop, King, Knight, Pawn, Queen, Rook};
use crate::chess::types::piece::BasePiece;
use crate::chess::types::square::Square;
use crate::general::bits;

pub fn see(first_attacker_square: Square, square: Square, board: &Board) -> i16{

    let first_victim = board.piece_at(square);

    if !first_victim.is_piece() {
        return 0;
    }


    let mut occupancy = board.occupancy() & !first_attacker_square.mask();

    let mut gain: i16 = PIECE_VALUES[first_victim as usize];
    let mut occupied_by_piece = BasePiece::from(board.piece_at(first_attacker_square));
    let start_color = board.side_to_move();
    let mut side_to_move = !board.side_to_move();


    loop{
        let attacker_square;
        let piece;

        let pawn_attacks = MOVEMENT_MASKS.pawn_attacks(!side_to_move, square) & board.bitboard(Pawn, side_to_move) & occupancy;
        let knight_attacks = MOVEMENT_MASKS.knight[square as usize] & board.bitboard(Knight, side_to_move) & occupancy;
        let bishop_attacks = bishop_lookup(square, occupancy) & board.bitboard(Bishop, side_to_move) & occupancy;
        let rook_attacks = rook_lookup(square, occupancy) & board.bitboard(Rook, side_to_move) & occupancy;
        let queen_attacks = queen_lookup(square, occupancy) & board.bitboard(Queen, side_to_move) & occupancy;
        let king_attacks = MOVEMENT_MASKS.king[square as usize] & board.bitboard(King, side_to_move) & occupancy;

        if      pawn_attacks   != 0 { attacker_square = Square::from(bits::next(pawn_attacks));   piece = Pawn; }
        else if knight_attacks != 0 { attacker_square = Square::from(bits::next(knight_attacks)); piece = Knight; }
        else if bishop_attacks != 0 { attacker_square = Square::from(bits::next(bishop_attacks)); piece = Bishop; }
        else if rook_attacks   != 0 { attacker_square = Square::from(bits::next(rook_attacks));   piece = Rook; }
        else if queen_attacks  != 0 { attacker_square = Square::from(bits::next(queen_attacks));  piece = Queen; }
        else if king_attacks   != 0 { attacker_square = Square::from(bits::next(king_attacks));   piece = King; }
        else { break }

        if side_to_move == start_color { gain += PIECE_VALUES[occupied_by_piece as usize]; }
        else                           { gain -= PIECE_VALUES[occupied_by_piece as usize]; }

        occupied_by_piece = piece;
        occupancy &= !attacker_square.mask();
        side_to_move = !side_to_move;

    }

    gain

}