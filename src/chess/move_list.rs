use crate::chess::move_ply::MovePly;
use crate::chess::types::move_flag::MoveFlag;
use crate::chess::types::square::Square;
use crate::general::bits;
use std::cmp::Reverse;
use std::slice::Iter;

const IS_BULK: u8 = 0b00000010;
const IS_PROMOTION: u8 = 0b00000001;
const FLAG_SHIFT: u8 = 2;

#[derive(Clone, Copy)]
pub struct PieceMoves {
    from_mask: u64,
    to_mask: u64,
    piece_move_info: u8,
}

impl PieceMoves {
    pub fn new_bulk(to_mask: u64, from_mask: u64, flag: MoveFlag) -> PieceMoves {
        PieceMoves {
            from_mask,
            to_mask,
            piece_move_info: IS_BULK | ((flag as u8) << FLAG_SHIFT),
        }
    }

    pub fn new(from: Square, to_mask: u64, flag: MoveFlag) -> PieceMoves {
        PieceMoves {
            from_mask: from.mask(),
            to_mask,
            piece_move_info: ((flag as u8) << FLAG_SHIFT),
        }
    }

    pub fn new_promote(to_mask: u64, from_mask: u64) -> PieceMoves {
        PieceMoves {
            from_mask,
            to_mask,
            piece_move_info: IS_BULK
                | IS_PROMOTION
                | ((MoveFlag::PromoteToKnight as u8) << FLAG_SHIFT),
        }
    }

    fn is_bulk(&self) -> bool {
        self.piece_move_info & IS_BULK != 0
    }

    fn is_promotion(&self) -> bool {
        self.piece_move_info & IS_PROMOTION != 0
    }

    fn flag(&self) -> MoveFlag {
        (self.piece_move_info >> FLAG_SHIFT).into()
    }

    fn update_promote_flag(&mut self) {
        let new_flag = (self.piece_move_info >> FLAG_SHIFT) + 1;
        self.piece_move_info &= IS_BULK | IS_PROMOTION;
        self.piece_move_info |= new_flag << FLAG_SHIFT;
    }

    fn reset_promote_flag(&mut self) {
        self.piece_move_info &= IS_BULK | IS_PROMOTION;
        self.piece_move_info |= (MoveFlag::PromoteToKnight as u8) << FLAG_SHIFT;
    }

    pub fn move_count(&self) -> u64 {
        let raw_count = self.to_mask.count_ones() as u64;
        if self.is_promotion() {
            return raw_count * 4;
        }
        raw_count
    }
}

impl Iterator for PieceMoves {
    type Item = MovePly;

    fn next(&mut self) -> Option<Self::Item> {
        if self.to_mask != 0 {
            let to = bits::next(self.to_mask);
            let from = bits::next(self.from_mask);

            // dont want to pop the from mask if its a promotion
            let promotion_mask = if self.is_promotion() { u64::MAX } else { 0 };
            //  want to pop the from mask if its in bulk
            let bulk_mask = if self.is_bulk() { 0 } else { u64::MAX };

            self.from_mask &= promotion_mask | bulk_mask | (self.from_mask - 1);

            let pop_to_mask = (self.is_promotion() && self.flag() == MoveFlag::PromoteToQueen)
                || !self.is_promotion();
            self.to_mask &= (self.to_mask - 1) | (pop_to_mask as u64 - 1);

            let flag = self.flag();

            if self.is_promotion() {
                if pop_to_mask {
                    self.reset_promote_flag();
                    self.from_mask &= self.from_mask - 1;
                } else {
                    self.update_promote_flag();
                }
            }

            return Some(MovePly::new(from.into(), to.into(), flag));
        }

        None
    }
}

#[derive(Clone)]
pub struct MoveList {
    moves: [MovePly; 256],
    move_count: usize,
}

impl Default for MoveList {
    fn default() -> Self {
        MoveList {
            moves: unsafe { core::mem::MaybeUninit::zeroed().assume_init() },
            move_count: 0,
        }
    }
}

impl MoveList {
    pub fn add_piece_moves(&mut self, piece_moves: &mut PieceMoves) {
        for mv in piece_moves {
            self.moves[self.move_count] = mv;
            self.move_count += 1;
        }
    }

    // pub fn add_moves(&mut self, mut to_mask: u64, from: Square, move_flag: MoveFlag) {
    //     while to_mask != 0 {
    //         let to = Square::from(bits::next(to_mask));

    //         self.moves[self.move_count] = MovePly::new(from, to, move_flag);
    //         to_mask &= to_mask - 1;

    //         self.move_count += 1;
    //     }
    // }

    // pub fn add_enpassant_moves(&mut self, to: Square, mut from_mask: u64) {
    //     while from_mask != 0 {
    //         let from = Square::from(bits::next(from_mask));

    //         self.moves[self.move_count] = MovePly::new(from, to, MoveFlag::EnPassantCapture);
    //         from_mask &= from_mask - 1;

    //         self.move_count += 1;
    //     }
    // }
    // pub fn add_bulk_moves(&mut self, mut to_mask: u64, mut from_mask: u64, move_flag: MoveFlag) {
    //     while to_mask != 0 {
    //         let to = Square::from(bits::next(to_mask));
    //         let from = Square::from(bits::next(from_mask));

    //         self.moves[self.move_count] = MovePly::new(from, to, move_flag);
    //         to_mask &= to_mask - 1;
    //         from_mask &= from_mask - 1;

    //         self.move_count += 1;
    //     }
    // }

    // pub fn add_bulk_promotion_moves(&mut self, mut to_mask: u64, mut from_mask: u64) {
    //     while to_mask != 0 {
    //         let to = Square::from(bits::next(to_mask));
    //         let from = Square::from(bits::next(from_mask));

    //         self.moves[self.move_count] = MovePly::new(from, to, MoveFlag::PromoteToKnight);
    //         self.moves[self.move_count + 1] = MovePly::new(from, to, MoveFlag::PromoteToBishop);
    //         self.moves[self.move_count + 2] = MovePly::new(from, to, MoveFlag::PromoteToRook);
    //         self.moves[self.move_count + 3] = MovePly::new(from, to, MoveFlag::PromoteToQueen);

    //         self.move_count += 4;

    //         to_mask &= to_mask - 1;
    //         from_mask &= from_mask - 1;
    //     }
    // }

    pub fn move_count(&self) -> usize {
        self.move_count
    }

    pub fn move_at(&self, index: usize) -> MovePly {
        self.moves[index]
    }

    pub fn iter(&self) -> Iter<'_, MovePly> {
        self.moves[..self.move_count].iter()
    }

    pub fn order_moves(&mut self, orderings: &[i16; 256]) {
        let mut indices = [0usize; 256];
        for i in 0..self.move_count {
            indices[i] = i;
        }

        indices[..self.move_count].sort_unstable_by_key(|&i| Reverse(orderings[i]));

        let original_copy = self.moves;

        for (j, &i) in indices[..self.move_count].iter().enumerate() {
            self.moves[j] = original_copy[i];
        }
    }

    pub fn contains_move(&self, checked_move: MovePly) -> bool {
        for cur_move in self.iter() {
            if *cur_move == checked_move {
                return true;
            }
        }

        false
    }
}
