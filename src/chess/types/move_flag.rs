use std::mem;
use crate::chess::types::color::Color;
use crate::chess::types::piece::Piece;

#[derive(Copy, Clone)]
#[repr(u8)]
pub enum MoveFlag {
    None,
    PromoteToKnight,
    PromoteToBishop,
    PromoteToRook,
    PromoteToQueen,
    DoubleJump,
    EnPassantCapture,
    CastleQueenSide,
    CastleKingSide,
}

impl From<u8> for MoveFlag{
    fn from(val: u8) -> MoveFlag{
        unsafe { mem::transmute(val) }
    }
}

impl MoveFlag {
    pub fn promotion_piece<const IS_BASE_PIECE: bool>(&self, color: Color) -> Piece{
        // the binary representation of white pieces and move flag promotion pieces are the same
        // so to convert to black pieces one can simply add 6
        if IS_BASE_PIECE {
            return unsafe{ mem::transmute(*self as u8 + 12) };
        }

        match color {
            Color::White => unsafe{ mem::transmute(*self as u8) },
            Color::Black => unsafe{ mem::transmute(*self as u8 +6) },
        }

    }
}
