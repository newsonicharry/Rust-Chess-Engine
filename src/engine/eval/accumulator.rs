use crate::chess::consts::NUM_SQUARES;
use crate::chess::types::piece::Piece;
use crate::chess::types::square::Square;
use crate::engine::eval::network::{*};

#[derive(Copy, Clone)]
pub struct Accumulator{
    pub white: [i16; HIDDEN_SIZE],
    pub black: [i16; HIDDEN_SIZE],
}


impl Default for Accumulator {
    fn default() -> Self {
        Self{
            white: MODEL.feature_biases,
            black: MODEL.feature_biases
        }
    }
}


impl Accumulator {
    fn nnue_index(piece: Piece, square: Square) -> (usize, usize) {
        let white_idx = NUM_SQUARES * piece as usize + square as usize;
        let black_idx = NUM_SQUARES * ((piece as usize + 6) % 12) + square.vert_flip() as usize;

        (white_idx * HIDDEN_SIZE, black_idx * HIDDEN_SIZE)
    }

    pub fn remove_piece(&mut self, piece: Piece, square: Square)  {
        let (white_idx, black_idx) = Self::nnue_index(piece, square);

        for i in 0..HIDDEN_SIZE {
            self.white[i] -= MODEL.feature_weights[i + white_idx];
            self.black[i] -= MODEL.feature_weights[i + black_idx];
        }

    }
    pub fn add_piece(&mut self, piece: Piece, square: Square)  {

        let (white_idx, black_idx) = Self::nnue_index(piece, square);

        for i in 0..HIDDEN_SIZE {
            self.white[i] += MODEL.feature_weights[i + white_idx];
            self.black[i] += MODEL.feature_weights[i + black_idx];
        }
    }

    pub fn move_piece(&mut self, piece: Piece, from: Square, to: Square)  {
        let (white_from_idx, black_from_idx) = Self::nnue_index(piece, from);
        let (white_to_idx, black_to_idx) = Self::nnue_index(piece, to);

        for i in 0..HIDDEN_SIZE {
            self.white[i] -= MODEL.feature_weights[i + white_from_idx];
            self.black[i] -= MODEL.feature_weights[i + black_from_idx];

            self.white[i] += MODEL.feature_weights[i + white_to_idx];
            self.black[i] += MODEL.feature_weights[i + black_to_idx];
        }
    }

    pub fn make_castle(&mut self, king: Piece, rook: Piece, king_from: Square, king_to: Square, rook_from: Square, rook_to: Square)  {
        let (king_white_from_idx, king_black_from_idx) = Self::nnue_index(king, king_from);
        let (king_white_to_idx, king_black_to_idx) = Self::nnue_index(king, king_to);

        let (rook_white_from_idx, rook_black_from_idx) = Self::nnue_index(rook, rook_from);
        let (rook_white_to_idx, rook_black_to_idx) = Self::nnue_index(rook, rook_to);

        for i in 0..HIDDEN_SIZE {
            self.white[i] -= MODEL.feature_weights[i + king_white_from_idx];
            self.black[i] -= MODEL.feature_weights[i + king_black_from_idx];

            self.white[i] += MODEL.feature_weights[i + king_white_to_idx];
            self.black[i] += MODEL.feature_weights[i + king_black_to_idx];

            self.white[i] -= MODEL.feature_weights[i + rook_white_from_idx];
            self.black[i] -= MODEL.feature_weights[i + rook_black_from_idx];

            self.white[i] += MODEL.feature_weights[i + rook_white_to_idx];
            self.black[i] += MODEL.feature_weights[i + rook_black_to_idx];
        }
    }

    pub fn make_capture(&mut self, piece: Piece, from: Square, to: Square, capture_piece: Piece, capture_square: Square)  {
        let (piece_white_from_idx, piece_black_from_idx) = Self::nnue_index(piece, from);
        let (piece_white_to_idx, piece_black_to_idx) = Self::nnue_index(piece, to);

        let (capture_white_idx, capture_black_idx) = Self::nnue_index(capture_piece, capture_square);

        for i in 0..HIDDEN_SIZE {
            self.white[i] -= MODEL.feature_weights[i + piece_white_from_idx];
            self.black[i] -= MODEL.feature_weights[i + piece_black_from_idx];

            self.white[i] += MODEL.feature_weights[i + piece_white_to_idx];
            self.black[i] += MODEL.feature_weights[i + piece_black_to_idx];

            self.white[i] -= MODEL.feature_weights[i + capture_white_idx];
            self.black[i] -= MODEL.feature_weights[i + capture_black_idx];
        }
    }

    pub fn make_promotion(&mut self, pawn: Piece, promotion: Piece, from: Square, to: Square) {
        let (pawn_white_from_idx, pawn_black_from_idx) = Self::nnue_index(pawn, from);
        let (promotion_white_to_idx, promotion_black_to_idx) = Self::nnue_index(promotion, to);

        for i in 0..HIDDEN_SIZE {
            self.white[i] -= MODEL.feature_weights[i + pawn_white_from_idx];
            self.black[i] -= MODEL.feature_weights[i + pawn_black_from_idx];

            self.white[i] += MODEL.feature_weights[i + promotion_white_to_idx];
            self.black[i] += MODEL.feature_weights[i + promotion_black_to_idx];
        }
    }
}