use crate::chess::move_ply::MovePly;
use crate::chess::types::file::File;
use crate::chess::types::piece::Piece;

#[derive(Copy, Clone)]
pub struct BoardState{
    pub played: MovePly,
    pub captured: Piece,
    pub half_move_clock: u8,
    pub castling_rights: u8,
    pub en_passant_file: File,
    pub can_en_passant: bool,
}