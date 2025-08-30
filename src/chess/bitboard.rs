use std::fmt::{Display, Formatter};
use std::ops::{BitAnd, BitOr, Not, BitAndAssign, BitOrAssign};
use crate::chess::types::square::Square;

#[derive(Copy)]
#[derive(Clone)]
pub struct Bitboard(pub u64);

impl Default for Bitboard {
    fn default() -> Self {
        Bitboard(0)
    }
}

impl Bitboard {

    pub fn move_piece(&mut self, from: Square, to: Square) {
        self.remove_piece(from);
        self.add_piece(to);
    }
    pub fn add_piece(&mut self, square: Square) {
        self.0 |= square.mask();
    }
    pub fn remove_piece(&mut self, square: Square) {
        self.0 ^= square.mask();
    }
}

impl BitOr<u64> for Bitboard {
    type Output = u64;
    fn bitor(self, rhs: u64) -> u64 { self.0 | rhs }
}

impl BitAnd<u64> for Bitboard {
    type Output = u64;
    fn bitand(self, rhs: u64) -> u64 { self.0 & rhs }
}

impl Not for Bitboard {
    type Output = u64;
    fn not(self) -> u64 { !self.0 }
}

impl BitOrAssign<u64> for Bitboard {
    fn bitor_assign(&mut self, rhs: u64) { self.0 |= rhs }
}

impl BitAndAssign<u64> for Bitboard {
    fn bitand_assign(&mut self, rhs: u64) { self.0 &= rhs }
}


impl Display for Bitboard {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}