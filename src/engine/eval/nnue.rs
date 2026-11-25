use crate::chess::bitboard::Bitboard;
use crate::chess::board::Board;
use crate::chess::consts::{MAX_MOVES, NUM_PIECES};
use crate::chess::move_ply::MovePly;
use crate::chess::types::color::Color;
use crate::chess::types::move_flag::MoveFlag;
use crate::chess::types::move_flag::MoveFlag::{CastleLong, CastleShort};
use crate::chess::types::piece::Piece::{BlackKing, BlackRook, WhiteKing, WhiteRook};
use crate::chess::types::square::Square;
use crate::engine::eval::accumulator::Accumulator;
use crate::engine::eval::network::*;

#[derive(Clone, Copy)]
pub struct CachedEntry {
    pub accumulator: [i16; HIDDEN_SIZE],
    pub pieces: [Bitboard; NUM_PIECES],
}

impl Default for CachedEntry {
    fn default() -> Self {
        Self {
            accumulator: MODEL.l0b,
            pieces: [Bitboard(0); NUM_PIECES],
        }
    }
}

#[derive(Copy, Clone, Default)]
struct CacheTable {
    pub table: [[[CachedEntry; INPUT_BUCKETS]; 2]; 2],
}

#[derive(Copy, Clone)]
pub struct NNUE {
    accumulator_stack: [Accumulator; MAX_MOVES / 2],
    curr_index: usize,
    cache: CacheTable,
}

impl NNUE {
    pub fn new(board: &Board) -> Self {
        let curr_accumulator = Accumulator::new(board);

        let mut cache_table = CacheTable::default();

        let white_flip_index = match curr_accumulator.white_flip {
            7 => 1usize,
            _ => 0usize,
        };

        let black_flip_index = match curr_accumulator.black_flip {
            7 => 1usize,
            _ => 0usize,
        };

        cache_table.table[Color::White as usize][white_flip_index][curr_accumulator.white_bucket] =
            CachedEntry {
                accumulator: curr_accumulator.white,
                pieces: board.all_bitboards(),
            };

        cache_table.table[Color::Black as usize][black_flip_index][curr_accumulator.black_bucket] =
            CachedEntry {
                accumulator: curr_accumulator.black,
                pieces: board.all_bitboards(),
            };

        let mut nnue = Self {
            accumulator_stack: [Accumulator::default(); MAX_MOVES / 2],
            curr_index: 0,
            cache: cache_table,
        };
        nnue.accumulator_stack[nnue.curr_index] = curr_accumulator;

        nnue
    }

    pub fn make_move(&mut self, played: &MovePly, board: &Board) {
        self.curr_index += 1;
        self.accumulator_stack[self.curr_index] = self.accumulator_stack[self.curr_index - 1];

        let curr_accumulator = &mut self.accumulator_stack[self.curr_index];

        let move_flag = played.flag();
        let from = played.from();
        let to = played.to();

        let piece = board.piece_at(from);
        let capture = board.piece_at(to);

        if INPUT_BUCKETS != 1 && piece.is_king() {
            let (prev_king_bucket, prev_flip, prev_accum) = match board.side_to_move() {
                Color::White => (
                    curr_accumulator.white_bucket,
                    curr_accumulator.white_flip,
                    &curr_accumulator.white,
                ),
                Color::Black => (
                    curr_accumulator.black_bucket,
                    curr_accumulator.black_flip,
                    &curr_accumulator.black,
                ),
            };

            let (new_king_bucket, new_flip) = get_king_bucket(to, board.side_to_move());

            if prev_king_bucket != new_king_bucket || prev_flip != new_flip {
                let prev_flip_index = match prev_flip {
                    7 => 1usize,
                    _ => 0usize,
                };

                self.cache.table[board.side_to_move() as usize][prev_flip_index]
                    [prev_king_bucket] = CachedEntry {
                    accumulator: *prev_accum,
                    pieces: board.all_bitboards(),
                };

                let new_flip_index = match new_flip {
                    7 => 1usize,
                    _ => 0usize,
                };
                let cached = &self.cache.table[board.side_to_move() as usize][new_flip_index]
                    [new_king_bucket];

                curr_accumulator.refresh(
                    cached,
                    board.side_to_move(),
                    new_king_bucket,
                    new_flip,
                    board,
                );
            }
        }

        if move_flag == MoveFlag::None {
            match capture.is_piece() {
                true => curr_accumulator.make_capture(piece, from, to, capture, to),
                false => curr_accumulator.move_piece(piece, from, to),
            }
        } else if move_flag == MoveFlag::DoubleJump {
            curr_accumulator.move_piece(piece, from, to);
        } else if move_flag == CastleShort {
            match board.side_to_move() {
                Color::White => {
                    curr_accumulator.make_castle(
                        WhiteKing,
                        WhiteRook,
                        Square::E1,
                        Square::G1,
                        Square::H1,
                        Square::F1,
                    );
                }
                Color::Black => {
                    curr_accumulator.make_castle(
                        BlackKing,
                        BlackRook,
                        Square::E8,
                        Square::G8,
                        Square::H8,
                        Square::F8,
                    );
                }
            }
        } else if move_flag == CastleLong {
            match board.side_to_move() {
                Color::White => {
                    curr_accumulator.make_castle(
                        WhiteKing,
                        WhiteRook,
                        Square::E1,
                        Square::C1,
                        Square::A1,
                        Square::D1,
                    );
                }
                Color::Black => {
                    curr_accumulator.make_castle(
                        BlackKing,
                        BlackRook,
                        Square::E8,
                        Square::C8,
                        Square::A8,
                        Square::D8,
                    );
                }
            }
        } else if move_flag.is_promotion() {
            let promotion_piece = move_flag.promotion_piece(board.side_to_move());
            if capture.is_piece() {
                curr_accumulator.remove_piece(capture, to);
            }
            curr_accumulator.make_promotion(piece, promotion_piece, from, to);
        } else if move_flag == MoveFlag::EnPassantCapture {
            let enemy_pawn_square = match board.side_to_move() {
                Color::White => Square::from(board.en_passant_file().unwrap() as u8 + 32),
                Color::Black => Square::from(board.en_passant_file().unwrap() as u8 + 24),
            };

            let enemy_pawn = board.piece_at(enemy_pawn_square);

            curr_accumulator.make_capture(piece, from, to, enemy_pawn, enemy_pawn_square);
        }
    }

    pub fn undo_move(&mut self) {
        self.curr_index -= 1;
    }

    // fn propagate_l2(&mut self) {
    //     self.l1 = MODEL.l1b[0];

    //     let white_accumulator = &self.accumulator_stack[self.cur_accumulator].white;
    //     for (i, (&value, &weight)) in white_accumulator.iter().zip(&MODEL.l1w).enumerate() {
    //         out += Self::squared_crelu(value) * weight as i32;
    //     }
    // }

    fn calc_output(value: i16, weight: i16) -> i32 {
        // identical logic, just keeps everything as an i16 when possible for performance
        let clamped = (value).clamp(CR_MIN as i16, CR_MAX as i16);
        let multiply = clamped.wrapping_mul(weight);
        multiply as i32 * clamped as i32
    }

    pub fn evaluate(&mut self, board: &Board) -> i16 {
        let white_accumulator = &self.accumulator_stack[self.curr_index].white;
        let black_accumulator = &self.accumulator_stack[self.curr_index].black;

        let (us, them) = match board.side_to_move() {
            Color::White => (white_accumulator.iter(), black_accumulator.iter()),
            Color::Black => (black_accumulator.iter(), white_accumulator.iter()),
        };

        let mut output_bucket = 0;
        if OUTPUT_BUCKETS != 1 {
            let num_pieces = board.occupancy().count_ones();
            let divisor = 32usize.div_ceil(OUTPUT_BUCKETS);
            output_bucket = ((num_pieces as u8 - 2) / divisor as u8) as usize;
        }

        let mut out = 0;
        for (&value, &weight) in us.zip(&MODEL.l1w[output_bucket][0]) {
            out += Self::calc_output(value, weight);
        }
        for (&value, &weight) in them.zip(&MODEL.l1w[output_bucket][1]) {
            out += Self::calc_output(value, weight);
        }

        out /= QA;
        out += MODEL.l1b[output_bucket] as i32;
        out = ((out * EVAL_SCALE) / QAB as i32).clamp(-28000, 28000);
        out as i16
    }
}
