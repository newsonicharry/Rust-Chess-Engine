use crate::chess::types::file::File;
use crate::chess::types::rank::Rank;
use std::fmt::Display;
use std::mem;
use std::ops::Rem;

#[repr(u8)]
#[derive(Clone,Copy)]
pub enum Square{
    A1, B1, C1, D1, E1, F1, G1, H1,
    A2, B2, C2, D2, E2, F2, G2, H2,
    A3, B3, C3, D3, E3, F3, G3, H3,
    A4, B4, C4, D4, E4, F4, G4, H4,
    A5, B5, C5, D5, E5, F5, G5, H5,
    A6, B6, C6, D6, E6, F6, G6, H6,
    A7, B7, C7, D7, E7, F7, G7, H7,
    A8, B8, C8, D8, E8, F8, G8, H8
}

impl Default for Square{
    fn default() -> Self {
        Square::A1
    }
}

impl From<u8> for Square{
    fn from(square: u8) -> Self {
        unsafe { mem::transmute(square) }
    }
}

impl From<String> for Square{
    fn from(square: String) -> Self {
        let file = File::from(square.chars().collect::<Vec<_>>()[0].to_string());
        let rank = Rank::from(square.chars().collect::<Vec<_>>()[1].to_string());
        
        Square::from((file, rank))
    }
}

impl From<(File, Rank)> for Square{
    fn from(cords: (File, Rank)) -> Self {
        let (file, rank) = cords;
        let index = rank as u8 * 8 + file as u8;
        
        unsafe { mem::transmute(index) }
    }
}

impl Rem<u8> for Square {
    type Output = u8;

    fn rem(self, rhs: u8) -> u8 {
        self as u8 % rhs
    }
}


impl PartialEq<u8> for Square {
    fn eq(&self, other: &u8) -> bool {
        *self as u8 == *other
    }
}

impl PartialEq<Self> for Square {
    fn eq(&self, other: &Self) -> bool {
        *self as u8 == *other as u8
    }
}

impl Square{
    pub fn mask(&self) -> u64{ 1 << *self as usize }

    pub fn file(&self) -> File { File::from( (*self as u8) % 8 ) }

    pub fn rank(&self) -> Rank { Rank::from( (*self as u8) / 8 ) }

    pub fn vert_flip(&self) -> u8{ *self as u8 ^ 56 }

}

impl Display for Square{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {

        let piece_dictionary = ["a1", "b1", "c1", "d1", "e1", "f1", "g1", "h1",
                                         "a2", "b2", "c2", "d2", "e2", "f2", "g2", "h2",
                                         "a3", "b3", "c3", "d3", "e3", "f3", "g3", "h3",
                                         "a4", "b4", "c4", "d4", "e4", "f4", "g4", "h4",
                                         "a5", "b5", "c5", "d5", "e5", "f5", "g5", "h5",
                                         "a6", "b6", "c6", "d6", "e6", "f6", "g6", "h6",
                                         "a7", "b7", "c7", "d7", "e7", "f7", "g7", "h7",
                                         "a8", "b8", "c8", "d8", "e8", "f8", "g8", "h8"];

        write!(f, "{}", piece_dictionary[*self as usize])


    }
}