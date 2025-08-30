use std::fmt::Display;

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
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
    NoPiece
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

            Piece::Pawn => 'P',
            Piece::Knight => 'N',
            Piece::Bishop => 'B',
            Piece::Rook => 'R',
            Piece::Queen => 'Q',
            Piece::King => 'K',

            Piece::NoPiece => ' '
        };

        write!(f, "{}", char)

    }
}


const ALL_PIECES: [Piece; 12] = [Piece::WhitePawn, Piece::WhiteKnight, Piece::WhiteBishop, Piece::WhiteRook, Piece::WhiteQueen, Piece::WhiteKing, Piece::BlackPawn, Piece::BlackKnight, Piece::BlackBishop, Piece::BlackRook, Piece::BlackQueen, Piece::BlackKing];
const WHITE_PIECES: [Piece; 6] = [Piece::WhitePawn, Piece::WhiteKnight, Piece::WhiteBishop, Piece::WhiteRook, Piece::WhiteQueen, Piece::WhiteKing];
const BLACK_PIECES: [Piece; 6] = [Piece::BlackPawn, Piece::BlackKnight, Piece::BlackBishop, Piece::BlackRook, Piece::BlackQueen, Piece::BlackKing];
const BASE_PIECES: [Piece; 6] = [Piece::Pawn, Piece::Knight, Piece::Bishop, Piece::Rook, Piece::Queen, Piece::King];


// impl Iterator

// impl Pieces{

// }


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