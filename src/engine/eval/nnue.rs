use crate::chess::board::Board;
use crate::chess::consts::MAX_MOVES;
use crate::chess::move_ply::MovePly;
use crate::chess::types::color::Color;
use crate::chess::types::move_flag::MoveFlag;
use crate::chess::types::move_flag::MoveFlag::{CastleShort, CastleLong};
use crate::chess::types::piece::Piece::{BlackKing, BlackRook, WhiteKing, WhiteRook};
use crate::chess::types::square::Square;
use crate::engine::eval::accumulator::Accumulator;
use crate::engine::eval::network::*;
use crate::general::bits;


#[derive(Copy, Clone)]
pub struct NNUE{
    accumulator_stack: [Accumulator; MAX_MOVES],
    cur_accumulator: usize,
}

impl Default for NNUE {
    fn default() -> Self {
        Self{
            accumulator_stack: [Accumulator::default(); MAX_MOVES],
            cur_accumulator: 0,
        }
    }
}

impl NNUE {
    pub fn new(&mut self, board: &mut Board){
        board.update_occupancy();
        let squares_with_pieces = bits::all_squares(board.occupancy());
        for square in squares_with_pieces {
            let piece = board.piece_at(square);
            self.accumulator_stack[self.cur_accumulator].add_piece(piece, square);
        }
    }
    

    pub fn make_move(&mut self, played: &MovePly, board: &Board){
        self.cur_accumulator += 1;
        self.accumulator_stack[self.cur_accumulator] = self.accumulator_stack[self.cur_accumulator-1];
        let current_accumulator = &mut self.accumulator_stack[self.cur_accumulator];

        let move_flag =  played.flag();
        let from = played.from();
        let to = played.to();

        let piece = board.piece_at(from);
        let capture= board.piece_at(to);


        if move_flag == MoveFlag::None{
            match capture.is_piece() {
                true => current_accumulator.make_capture(piece, from, to, capture, to),
                false => current_accumulator.move_piece(piece, from, to)
            }
        }
            
        else if move_flag == MoveFlag::DoubleJump {
            current_accumulator.move_piece(piece, from, to);
        }

        else if move_flag == CastleShort {
            match board.side_to_move() {
                Color::White => { current_accumulator.make_castle(WhiteKing, WhiteRook, Square::E1, Square::G1, Square::H1, Square::F1); }
                Color::Black => { current_accumulator.make_castle(BlackKing, BlackRook, Square::E8, Square::G8, Square::H8, Square::F8); }
            }
        }

        else if move_flag == CastleLong {
            match board.side_to_move() {
                Color::White => { current_accumulator.make_castle(WhiteKing, WhiteRook, Square::E1, Square::C1, Square::A1, Square::D1); }
                Color::Black => { current_accumulator.make_castle(BlackKing, BlackRook, Square::E8, Square::C8, Square::A8, Square::D8); }
            }
        }

        else if move_flag.is_promotion(){
            let promotion_piece = move_flag.promotion_piece(board.side_to_move());
            current_accumulator.make_promotion(piece, promotion_piece, from, to);
        }

        else if move_flag == MoveFlag::EnPassantCapture {
            let enemy_pawn_square = match board.side_to_move() {
                Color::White => Square::from(board.en_passant_file().unwrap() as u8 + 32),
                Color::Black => Square::from(board.en_passant_file().unwrap() as u8 + 24),
            };

            let enemy_pawn = board.piece_at(enemy_pawn_square);

            current_accumulator.make_capture(piece, from, to, enemy_pawn, enemy_pawn_square);
        }


    }

    pub fn undo_move(&mut self){
        self.cur_accumulator -= 1;
    }
    
    fn squared_crelu(value: i16) -> i32 {
        (value as i32).clamp(CR_MIN, CR_MAX).pow(2)
    }
    pub fn evaluate(&self, side_to_move: Color) -> i16 {
        let white_accumulator = &self.accumulator_stack[self.cur_accumulator].white;
        let black_accumulator = &self.accumulator_stack[self.cur_accumulator].black;

        let (us, them) = match side_to_move {
            Color::White => (white_accumulator.iter(), black_accumulator.iter()),
            Color::Black => (black_accumulator.iter(), white_accumulator.iter()),
        };

        let mut out = 0;
        for (&value, &weight) in us.zip(&MODEL.output_weights[..HIDDEN_SIZE]) {
            out += Self::squared_crelu(value) * weight as i32;
        }
        for (&value, &weight) in them.zip(&MODEL.output_weights[HIDDEN_SIZE..]) {
            out += Self::squared_crelu(value) * weight as i32;
        }

        ((out / QA + MODEL.output_bias as i32) * EVAL_SCALE / QAB) as i16
    }
}


