use std::fmt::Display;
use std::mem;
use std::ops::Not;

#[repr(u8)]
#[derive(Copy, Clone, PartialEq)]
pub enum Color {
    Black,
    White,
}

impl From<bool> for Color {
    fn from(side_to_move: bool) -> Self {
        unsafe { mem::transmute(side_to_move) }
    }
}

impl Not for Color {
    type Output = Color;
    fn not(self) -> Self::Output {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }
}

impl Color {
    pub fn is_white(self) -> bool {
        match self {
            Color::White => true,
            Color::Black => false,
        }
    }
}

impl Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let final_str = match self {
            Color::White => "White",
            Color::Black => "Black",
        };

        write!(f, "{}", final_str)
    }
}
