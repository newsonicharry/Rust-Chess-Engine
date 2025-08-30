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

impl Square{
    pub fn mask(&self) -> u64{ 1 << *self as usize }

    pub fn file(&self) -> File { File::from( (*self as u8) % 8 ) }

    pub fn rank(&self) -> Rank { Rank::from( (*self as u8) / 8 ) }

    pub fn vert_flip(&self) -> u8{ *self as u8 ^ 56 }
}

impl Display for Square{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {

        let piece_dictionary = ["A1", "B1", "C1", "D1", "E1", "F1", "G1", "H1",
                                         "A2", "B2", "C2", "D2", "E2", "F2", "G2", "H2",
                                         "A3", "B3", "C3", "D3", "E3", "F3", "G3", "H3",
                                         "A4", "B4", "C4", "D4", "E4", "F4", "G4", "H4",
                                         "A5", "B5", "C5", "D5", "E5", "F5", "G5", "H5",
                                         "A6", "B6", "C6", "D6", "E6", "F6", "G6", "H6",
                                         "A7", "B7", "C7", "D7", "E7", "F7", "G7", "H7",
                                         "A8", "B8", "C8", "D8", "E8", "F8", "G8", "H8"];

        write!(f, "{}", piece_dictionary[*self as usize])


    }
}