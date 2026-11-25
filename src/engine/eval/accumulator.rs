use crate::chess::board::Board;
use crate::chess::consts::NUM_SQUARES;
use crate::chess::types::color::Color;
use crate::chess::types::piece::{ITER_ALL, Piece};
use crate::chess::types::square::Square;
use crate::engine::eval::network::*;
use crate::engine::eval::nnue::CachedEntry;
use crate::general::bits;

#[derive(Copy, Clone)]
pub struct Accumulator {
    pub white: [i16; HIDDEN_SIZE],
    pub black: [i16; HIDDEN_SIZE],
    pub white_bucket: usize,
    pub black_bucket: usize,
    pub white_flip: u8,
    pub black_flip: u8,
}

impl Default for Accumulator {
    fn default() -> Self {
        Self {
            white: MODEL.l0b,
            black: MODEL.l0b,
            white_bucket: 0,
            black_bucket: 0,
            white_flip: 0,
            black_flip: 0,
        }
    }
}

const ADD: bool = true;
const SUB: bool = false;

impl Accumulator {
    pub fn new(board: &Board) -> Accumulator {
        let mut accumulator = Accumulator::default();

        if INPUT_BUCKETS != 1 {
            (accumulator.white_bucket, accumulator.white_flip) =
                get_king_bucket(board.king_square(Color::White), Color::White);
            (accumulator.black_bucket, accumulator.black_flip) =
                get_king_bucket(board.king_square(Color::Black), Color::Black);
        }

        let squares_with_pieces = bits::all_squares(board.occupancy());
        for square in squares_with_pieces {
            let piece = board.piece_at(square);
            accumulator.add_piece(piece, square);
        }

        accumulator
    }

    pub fn refresh(
        &mut self,
        cache: &CachedEntry,
        side: Color,
        new_bucket: usize,
        new_flip: u8,
        board: &Board,
    ) {
        match side {
            Color::White => {
                self.white = cache.accumulator;
                self.white_bucket = new_bucket;
                self.white_flip = new_flip;
            }
            Color::Black => {
                self.black = cache.accumulator;
                self.black_bucket = new_bucket;
                self.black_flip = new_flip;
            }
        };

        for piece in Piece::iterator::<ITER_ALL>() {
            let saved = cache.pieces[piece as usize];
            let curr = board.bitboard_piece(piece);
            let mut add = curr & !saved;
            let mut sub = saved & !curr;

            while add != 0 {
                let index = add.trailing_zeros();
                self.modify_sided_piece::<ADD>(side, piece, index.into());
                add &= !(1u64 << index);
            }

            while sub != 0 {
                let index = sub.trailing_zeros();
                self.modify_sided_piece::<SUB>(side, piece, index.into());
                sub &= !(1u64 << index);
            }
        }
    }

    fn nnue_index(&self, piece: Piece, square: Square) -> (usize, usize) {
        let white_idx = NUM_SQUARES * piece as usize + square as usize ^ self.white_flip as usize;

        let mut reversed_color = piece as u8;
        if reversed_color >= 6 {
            reversed_color -= 6;
        } else {
            reversed_color += 6;
        }

        let black_idx = NUM_SQUARES * reversed_color as usize + square.vert_flip() as usize
            ^ self.black_flip as usize;

        (white_idx * HIDDEN_SIZE, black_idx * HIDDEN_SIZE)
    }

    #[inline(always)]
    fn update_weights<const SIGN: bool>(
        accumulator: &mut [i16; HIDDEN_SIZE],
        weights: &[i16; HIDDEN_SIZE],
    ) {
        for (value, weight) in accumulator.iter_mut().zip(weights) {
            match SIGN {
                ADD => *value += *weight,
                SUB => *value -= *weight,
            }
        }
    }

    #[inline(always)]
    fn update_accumulator<const SIGN: bool>(
        accum: &mut [i16; HIDDEN_SIZE],
        bucket: usize,
        idx: usize,
    ) {
        Self::update_weights::<SIGN>(
            accum,
            (&MODEL.l0w[bucket][idx..idx + HIDDEN_SIZE])
                .try_into()
                .unwrap(),
        );
    }

    pub fn remove_piece(&mut self, piece: Piece, square: Square) {
        let (white_idx, black_idx) = self.nnue_index(piece, square);

        Self::update_accumulator::<SUB>(&mut self.white, self.white_bucket, white_idx);
        Self::update_accumulator::<SUB>(&mut self.black, self.black_bucket, black_idx);
    }

    pub fn add_piece(&mut self, piece: Piece, square: Square) {
        let (white_idx, black_idx) = self.nnue_index(piece, square);

        Self::update_accumulator::<ADD>(&mut self.white, self.white_bucket, white_idx);
        Self::update_accumulator::<ADD>(&mut self.black, self.black_bucket, black_idx);
    }

    fn modify_sided_piece<const SIGN: bool>(&mut self, side: Color, piece: Piece, square: Square) {
        let (white_idx, black_idx) = self.nnue_index(piece, square);

        match side {
            Color::White => {
                Self::update_accumulator::<SIGN>(&mut self.white, self.white_bucket, white_idx)
            }
            Color::Black => {
                Self::update_accumulator::<SIGN>(&mut self.black, self.black_bucket, black_idx)
            }
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
