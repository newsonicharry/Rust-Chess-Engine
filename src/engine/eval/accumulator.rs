use crate::chess::consts::NUM_SQUARES;
use crate::chess::types::piece::Piece;
use crate::chess::types::square::Square;
use crate::engine::eval::network::*;

#[derive(Copy, Clone)]
pub struct Accumulator {
    pub white: [i16; HIDDEN_SIZE],
    pub black: [i16; HIDDEN_SIZE],
}

impl Default for Accumulator {
    fn default() -> Self {
        Self {
            white: MODEL.feature_biases,
            black: MODEL.feature_biases,
        }
    }
}

impl Accumulator {
    fn nnue_index(piece: Piece, square: Square) -> (usize, usize) {
        let white_idx = NUM_SQUARES * piece as usize + square as usize;

        let mut reversed_color = piece as u8;
        if reversed_color >= 6 {
            reversed_color -= 6;
        } else {
            reversed_color += 6;
        }

        let black_idx = NUM_SQUARES * reversed_color as usize + square.vert_flip() as usize;

        (white_idx * HIDDEN_SIZE, black_idx * HIDDEN_SIZE)
    }

    pub fn remove_piece(&mut self, piece: Piece, square: Square) {
        let (white_idx, black_idx) = Self::nnue_index(piece, square);

        for i in 0..HIDDEN_SIZE {
            self.white[i] -= MODEL.feature_weights[i + white_idx];
            self.black[i] -= MODEL.feature_weights[i + black_idx];
        }
    }
    pub fn add_piece(&mut self, piece: Piece, square: Square) {
        let (white_idx, black_idx) = Self::nnue_index(piece, square);

        for i in 0..HIDDEN_SIZE {
            self.white[i] += MODEL.feature_weights[i + white_idx];
            self.black[i] += MODEL.feature_weights[i + black_idx];
        }
    }

    pub fn move_piece(&mut self, piece: Piece, from: Square, to: Square) {
        self.add_piece(piece, to);
        self.remove_piece(piece, from);
    }

    pub fn make_castle(
        &mut self,
        king: Piece,
        rook: Piece,
        king_from: Square,
        king_to: Square,
        rook_from: Square,
        rook_to: Square,
    ) {
        self.move_piece(king, king_from, king_to);
        self.move_piece(rook, rook_from, rook_to);
    }

    pub fn make_capture(
        &mut self,
        piece: Piece,
        from: Square,
        to: Square,
        capture_piece: Piece,
        capture_square: Square,
    ) {
        self.move_piece(piece, from, to);
        self.remove_piece(capture_piece, capture_square);
    }

    pub fn make_promotion(&mut self, pawn: Piece, promotion: Piece, from: Square, to: Square) {
        self.remove_piece(pawn, from);
        self.add_piece(promotion, to);
    }
}
