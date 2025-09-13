use std::fmt::{Display, Formatter};
use crate::chess::bitboard::Bitboard;

#[repr(u8)]
#[derive(Copy, Clone)]
#[derive(PartialEq)]
pub enum File{
    A, B, C, D, E, F, G, H
}

impl From<u8> for File {
    fn from(file: u8) -> Self {
        unsafe { std::mem::transmute(file) }
    }
}

impl From<String> for File {
    fn from(file: String) -> Self {

        match file.as_str() {
            "a" => File::A,
            "b" => File::B,
            "c" => File::C,
            "d" => File::D,
            "e" => File::E,
            "f" => File::F,
            "g" => File::G,
            "h" => File::H,
            _ => unreachable!()
        }
    }
}

impl Default for File{
    fn default() -> Self {
        File::A
    }
}

impl File {
    pub fn add(&mut self, amount: u8){
        *self = File::from(*self as u8 + amount);
    }

    pub fn sub(&mut self, amount: u8){
        *self = File::from(*self as u8 - amount);
    }
    pub fn plus(&mut self){
        self.add(1);
    }

    pub fn minus(&mut self){
        self.sub(1);
    }
}



impl Display for File {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {

        let file_as_char = match self {
            File::A => 'a',
            File::B => 'b',
            File::C => 'c',
            File::D => 'd',
            File::E => 'e',
            File::F => 'f',
            File::G => 'g',
            File::H => 'h'
        };

        write!(f,"{}", file_as_char)

    }
}

