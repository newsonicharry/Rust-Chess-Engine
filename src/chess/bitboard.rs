use std::fmt::{Display, Formatter};
use std::ops::{BitAnd, BitOr, Not, BitAndAssign, BitOrAssign};
use crate::chess::board::Board;
use crate::chess::consts::NUM_SQUARES;
use crate::chess::types::square::Square;

#[derive(Copy)]
#[derive(Clone)]
pub struct Bitboard(pub u64);

impl Default for Bitboard {
    fn default() -> Self {
        Bitboard(0)
    }
}

impl From<u64> for Bitboard {
    fn from(bits: u64) -> Self {
        Bitboard(bits)
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



const TOP_SECTION: &str    = "    ┌─────┬─────┬─────┬─────┬─────┬─────┬─────┬─────┐";
const MIDDLE_SECTION: &str = "    ├─────┼─────┼─────┼─────┼─────┼─────┼─────┼─────┤";
const BOTTOM_SECTION: &str = "    └─────┴─────┴─────┴─────┴─────┴─────┴─────┴─────┘";
const FILE_LABEL: &str =     "       a     b     c     d     e     f     g     h   ";
const SIDE_BAR: &str = "│";




impl Display for Bitboard {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {


        let mut pretty_print = TOP_SECTION.to_string();

        for i in 0..NUM_SQUARES {

            if i % 8 == 0 {

                if i != 0 {
                    pretty_print += &*(SIDE_BAR.to_owned() + "\n" + MIDDLE_SECTION + "\n");
                }
                else {
                    pretty_print += "\n";
                }

                pretty_print += &*(" ".to_owned() + &*((i ^ 56) / 8 + 1).to_string() + "  ");
            }

            let square = Square::from((i ^ 56) as u8);
            let bit_on = self.0 & square.mask() != 0;

            let item = match bit_on {
                true => '-',
                false => ' ',
            };

            pretty_print += &*(SIDE_BAR.to_owned() + "  " + &*item.to_string() + "  ");

        }

        pretty_print += &*(SIDE_BAR.to_owned() + "\n" + BOTTOM_SECTION + "\n" + FILE_LABEL + "\n");

        write!(f, "{}", pretty_print)

    }
}