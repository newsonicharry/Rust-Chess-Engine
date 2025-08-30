use std::fmt::Display;
use std::ops::Not;

#[repr(u8)]
#[derive(Copy, Clone)]
pub enum Color{
    White,
    Black,
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

impl Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let final_str = match self { 
            Color::White => "White",
            Color::Black => "Black",
        };
        
        write!(f, "{}", final_str)
        
    }
}
