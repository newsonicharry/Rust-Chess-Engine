use std::fmt::Display;
use std::mem;
use crate::chess::types::color::Color;
use crate::chess::types::piece::Piece::{BlackKing, BlackPawn, NoPiece, WhiteKing, WhitePawn, WhiteRook};

#[derive(Copy, Clone)]
#[repr(u8)]
pub enum Piece {
    WhitePawn,
    WhiteKnight,
    WhiteBishop,
    WhiteRook,
    WhiteQueen,
    WhiteKing,
    BlackPawn,
    BlackKnight,
    BlackBishop,
    BlackRook,
    BlackQueen,
    BlackKing,
    NoPiece,
}


impl From<(BasePiece, Color)> for Piece {
    fn from(piece_data: (BasePiece, Color)) -> Self {
        let (base_piece, color) = piece_data;
        let raw_value = match color { 
            Color::White => base_piece as u8,
            Color::Black => base_piece as u8+6,
        };
        
        unsafe { mem::transmute(raw_value) }
        
    }
}

impl Display for Piece {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let char = match self {
            Piece::WhitePawn => 'P',
            Piece::WhiteKnight => 'N',
            Piece::WhiteBishop => 'B',
            Piece::WhiteRook => 'R',
            Piece::WhiteQueen => 'Q',
            Piece::WhiteKing => 'K',

            Piece::BlackPawn => 'p',
            Piece::BlackKnight => 'n',
            Piece::BlackBishop => 'b',
            Piece::BlackRook => 'r',
            Piece::BlackQueen => 'q',
            Piece::BlackKing => 'k',

            Piece::NoPiece => ' '
        };

        write!(f, "{}", char)

    }
}


const ALL_PIECES: [Piece; 12] = [Piece::WhitePawn, Piece::WhiteKnight, Piece::WhiteBishop, Piece::WhiteRook, Piece::WhiteQueen, Piece::WhiteKing, Piece::BlackPawn, Piece::BlackKnight, Piece::BlackBishop, Piece::BlackRook, Piece::BlackQueen, Piece::BlackKing];
const WHITE_PIECES: [Piece; 6] = [Piece::WhitePawn, Piece::WhiteKnight, Piece::WhiteBishop, Piece::WhiteRook, Piece::WhiteQueen, Piece::WhiteKing];
const BLACK_PIECES: [Piece; 6] = [Piece::BlackPawn, Piece::BlackKnight, Piece::BlackBishop, Piece::BlackRook, Piece::BlackQueen, Piece::BlackKing];


pub const ITER_ALL: u8 = 0;
pub const ITER_WHITE: u8 = 1;
pub const ITER_BLACK: u8 = 2;

impl Piece{
    pub fn iterator<const ITER_TYPE: u8>() -> impl Iterator<Item = Piece> {

        match ITER_TYPE {
            ITER_ALL => ALL_PIECES.iter().copied(),
            ITER_WHITE => WHITE_PIECES.iter().copied(),
            ITER_BLACK => BLACK_PIECES.iter().copied(),
            _ => unreachable!()
        }
    }
    
    pub fn is_piece(&self) -> bool {
        *self as u8 != NoPiece as u8
    }

    pub fn is_pawn(&self) -> bool {
        *self as u8 == WhitePawn as u8 || *self as u8 == BlackPawn as u8
    }
    
    pub fn is_king(&self) -> bool{
        *self as u8 == WhiteKing as u8 || *self as u8 == BlackKing as u8
    }
    
    pub fn color(&self) -> Color {
        if *self as u8 >= 6 { 
            return Color::Black
        }
        
        Color::White
    }
}



#[derive(Copy, Clone)]
#[repr(u8)]
pub enum BasePiece{
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}


const BASE_PIECES: [BasePiece; 6] = [BasePiece::Pawn, BasePiece::Knight, BasePiece::Bishop, BasePiece::Rook, BasePiece::Queen, BasePiece::King];


impl BasePiece{
    pub fn iterator() -> impl Iterator<Item = BasePiece> {
        BASE_PIECES.iter().copied()
    }
}

impl Display for BasePiece {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let char = match self{
            BasePiece::Pawn => 'P',
            BasePiece::Knight => 'N',
            BasePiece::Bishop => 'B',
            BasePiece::Rook => 'R',
            BasePiece::Queen => 'Q',
            BasePiece::King => 'K',
        };

        write!(f, "{}", char)

    }
}


pub fn char_to_piece(piece: char) -> Option<Piece>{
    match piece {
        'P' => Some(Piece::WhitePawn),
        'N' => Some(Piece::WhiteKnight),
        'B' => Some(Piece::WhiteBishop),
        'R' => Some(Piece::WhiteRook),
        'Q' => Some(Piece::WhiteQueen),
        'K' => Some(Piece::WhiteKing),
        
        'p' => Some(Piece::BlackPawn),
        'n' => Some(Piece::BlackKnight),
        'b' => Some(Piece::BlackBishop),
        'r' => Some(Piece::BlackRook),
        'q' => Some(Piece::BlackQueen),
        'k' => Some(Piece::BlackKing),
        
        _ => None
    }
}