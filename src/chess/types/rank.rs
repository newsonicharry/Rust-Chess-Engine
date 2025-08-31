use std::fmt::{Display, Formatter};
use crate::chess::types::color::Color;

#[derive(Copy, Clone)]
#[repr(u8)]
pub enum Rank{
    First,
    Second,
    Third,
    Fourth,
    Fifth,
    Sixth,
    Seventh,
    Eighth,
}

impl From<u8> for Rank{
    fn from(rank: u8) -> Self{
        unsafe { std::mem::transmute(rank) }
    }
}



impl Rank {
    pub fn add(&mut self, amount: u8){
        *self = Rank::from(*self as u8 + amount);
    }

    pub fn sub(&mut self, amount: u8){
        *self = Rank::from(*self as u8 - amount);
    }
    pub fn plus(&mut self){
        self.add(1);
    }

    pub fn minus(&mut self){
        self.sub(1);
    }

    pub fn is_pawn_start(&self, color: Color) -> bool{

        match color {
            Color::White => { *self as u8 == 1 }
            Color::Black => { *self as u8 == 6 }
        }

    }

    pub fn is_pawn_promotion(&self, color: Color) -> bool{

        match color {
            Color::White => { *self as u8 == 7 }
            Color::Black => { *self as u8 == 0 }
        }

    }

    pub fn can_pawn_promote(&self, color: Color) -> bool{
        match color {
            Color::White => { *self as u8 == 6 }
            Color::Black => { *self as u8 == 1 }
        }
    }
}


impl Display for Rank {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {

        let file_as_char = match self {
            Rank::First =>  '1',
            Rank::Second => '2',
            Rank::Third =>  '3',
            Rank::Fourth => '4',
            Rank::Fifth =>  '5',
            Rank::Sixth =>  '6',
            Rank::Seventh =>'7',
            Rank::Eighth => '8',
        };

        write!(f,"{}", file_as_char)

    }
}
