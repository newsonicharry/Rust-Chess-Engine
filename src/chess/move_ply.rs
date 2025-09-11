use std::cmp::PartialEq;
use crate::chess::types::move_flag::MoveFlag;
use crate::chess::types::square::Square;
use std::fmt::Display;
use crate::chess::board::Board;
use crate::chess::types::color::Color;
use crate::chess::types::color::Color::Black;
use crate::chess::types::square::Square::{C1, C8, E1, E8, G1, G8};

const SQUARE_MASK: u16 = 0b111111;
const TO_SHIFT: u8 = 6;
const FLAG_SHIFT: u8 = 12;

#[derive(Clone, Copy)]
pub struct MovePly{
    //   flag   to    from
    // 0b1111 111111 111111
    packed_data: u16
}

impl Default for MovePly{
    fn default() -> Self{
        MovePly{packed_data: 0}
    }
}

impl From<u16> for MovePly{
    fn from(data: u16) -> Self{
        MovePly{packed_data: data}
    }
}


impl MovePly{
    pub fn new(from: Square, to: Square, flag: MoveFlag) -> Self{
        let packed_data =  ((flag as u16) << FLAG_SHIFT)
                         | ((to as u16) << TO_SHIFT)
                         | from as u16;

        MovePly{ packed_data }
    }


    pub fn from(&self) -> Square{
        Square::from((self.packed_data & SQUARE_MASK) as u8)
    }

    pub fn to(&self) -> Square{
        Square::from(((self.packed_data >> TO_SHIFT) & SQUARE_MASK) as u8)
    }

    pub fn flag(&self) -> MoveFlag{
        MoveFlag::from((self.packed_data >> FLAG_SHIFT) as u8)
    }
    
    pub fn packed_data(&self) -> u16{
        self.packed_data
    }
}

impl Display for MovePly {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let from_file = self.from().file();
        let from_rank = self.from().rank();

        let to_file = self.to().file();
        let to_rank = self.to().rank();


        let mut promotion_piece = "".to_owned();

        let flag = self.flag();
        if flag.is_promotion() {
            promotion_piece = flag.promotion_piece(Black).to_string();
        }

        let final_str = from_file.to_string() + &*from_rank.to_string() + &*to_file.to_string() + &*to_rank.to_string() + &*promotion_piece;
        write!(f, "{}", final_str)

    }
}

pub fn uci_move_parser(played: String, board: &Board) -> MovePly{
    let from = Square::from(played[0..2].to_string());
    let to = Square::from(played[2..4].to_string());

    let mut flag = MoveFlag::None;

    if played.len() == 5{
        let promotion_piece = played[4..].to_string();
        match promotion_piece.as_str() {
            "n" => MoveFlag::PromoteToKnight,
            "b" => MoveFlag::PromoteToBishop,
            "r" => MoveFlag::PromoteToRook,
            "q" => MoveFlag::PromoteToQueen,
            _=> unreachable!(),
        };
    }

    let piece_type = board.piece_at(from);
    let piece_color = piece_type.color();

    if piece_type.is_pawn()  {
        if from.rank().is_pawn_start(piece_color) && to.rank().is_pawn_jump_end(piece_color) {
            flag = MoveFlag::DoubleJump;
        }

        if from.file() != to.file() && !board.piece_at(to).is_piece() {
            flag = MoveFlag::EnPassantCapture;
        }
    }


    if piece_type.is_king() {
        if (from == E1 && to == G1) || (from == E8 && to == G8){
            flag = MoveFlag::CastleShort;
        }

        if (from == E1 && to == C1) || (from == E8 && to == C8){
            flag = MoveFlag::CastleLong;
        }
    }


    MovePly::new(from, to, flag)
}