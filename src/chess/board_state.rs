use crate::chess::move_ply::MovePly;
use crate::chess::types::file::File;
use crate::chess::types::piece::Piece;
use crate::chess::types::piece::Piece::NoPiece;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct BoardState{
    pub played: MovePly,
    pub captured: Piece,
    pub half_move_clock: u8,
    pub castling_rights: u8,
    pub en_passant_file: File,
    pub can_en_passant: bool,
}

impl Default for BoardState {
    fn default() -> BoardState {
        BoardState{
            played: MovePly::default(),
            captured: NoPiece,
            half_move_clock: 0,
            castling_rights: 0,
            en_passant_file: File::default(),
            can_en_passant: false,
        }
    }
}