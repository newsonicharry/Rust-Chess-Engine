use crate::chess::move_ply::MovePly;
use crate::engine::types::tt_flag::TTFlag;
use portable_atomic::{AtomicU8, AtomicU32, AtomicU128};
use std::mem;
use std::sync::atomic::{AtomicI16, AtomicU16, Ordering};

// Type     Bits
// zobrist: 64
// curmove: 16
// eval   : 16
// depth  : 8
// score  : 8
// gen    : 8
// flag   : 2

#[derive(Copy, Clone, Default)]
pub struct TTEntry {
    pub zobrist: u64,
    pub cur_move: MovePly,
    pub eval: i16,
    pub depth: u8,
    pub score: u8,
    pub generation: u8,
    pub tt_flag: TTFlag,
}

impl From<u128> for TTEntry {
    fn from(packed_data: u128) -> Self {
        unsafe { mem::transmute(packed_data) }
    }
}

const AGE_MULTIPLIER: i16 = -1;
const EXACT_BONUS: i16 = 12;
const DEPTH_MULTIPLIER: i16 = 4;

const ENTRY_SIZE: usize = std::mem::size_of::<TTEntry>();

pub struct Transposition {
    entries: Box<[AtomicU128]>,
    pub generation: AtomicU8,

    pub curr_depth: AtomicU8,
    entries_filled: AtomicU32,
    pub best_move_score: AtomicI16,
    pub best_move: AtomicU16,
}

impl Transposition {
    pub fn new(mb_size: u16) -> Self {
        let size_as_bytes = mb_size as u64 * 1024 * 1024;
        let max_num_entries = size_as_bytes / ENTRY_SIZE as u64;

        let round_down_pow2 = 1 << (63 - max_num_entries.leading_zeros());

        let entries: Vec<AtomicU128> = (0..round_down_pow2).map(|_| AtomicU128::new(0)).collect();

        Self {
            entries: entries.into(),
            generation: AtomicU8::new(0),
            curr_depth: AtomicU8::new(0),
            entries_filled: AtomicU32::new(0),
            best_move: AtomicU16::new(0),
            best_move_score: AtomicI16::new(0),
        }
    }

    pub fn hash_full(&self) -> u32 {
        self.entries_filled.load(Ordering::Relaxed) * 1000 / self.entries.len() as u32
    }

    pub fn age(&self) {
        // dont want an overflow
        if self.generation.load(Ordering::Relaxed) == u8::MAX {
            return;
        }
        self.generation.add(1, Ordering::Relaxed);
    }

    pub fn probe(&self, zobrist: u64) -> Option<TTEntry> {
        let index = zobrist & (self.entries.len() as u64 - 1);
        let entry_data = self.entries[index as usize].load(Ordering::Relaxed);
        let tt_entry: TTEntry = unsafe { mem::transmute(entry_data) };
        if tt_entry.zobrist == zobrist {
            return Some(tt_entry);
        }
        None
    }

    pub fn update(
        &self,
        zobrist: u64,
        cur_move: MovePly,
        mut eval: i16,
        depth: u8,
        tt_flag: TTFlag,
        _is_pv: bool,
        ply: u8,
    ) {
        if eval > 29000 {
            eval += ply as i16;
        } else if eval < -29000 {
            eval -= ply as i16;
        }

        // calculate the new entry score
        let mut score = depth as i16 * DEPTH_MULTIPLIER as i16;
        match tt_flag {
            TTFlag::Exact => score += EXACT_BONUS,
            _ => {}
        }
        let score = score.clamp(0, 255) as u8;

        // new entry
        let new_entry = TTEntry {
            zobrist,
            cur_move,
            eval,
            depth,
            score,
            generation: self.generation.load(Ordering::Relaxed),
            tt_flag,
        };

        // load old entry
        let index = (zobrist & (self.entries.len() as u64 - 1)) as usize;
        let entry: TTEntry = unsafe { mem::transmute(self.entries[index].load(Ordering::Relaxed)) };

        let store_new_entry = || {
            self.entries[index].store(unsafe { mem::transmute(new_entry) }, Ordering::Relaxed);
        };

        // if its an empty entry its safe to replace
        if entry.zobrist == 0 {
            store_new_entry();
            self.entries_filled.add(1, Ordering::Relaxed);
            return;
        }

        // if its the same position but the new entry has a higher dpeth
        if entry.zobrist == new_entry.zobrist && entry.depth < new_entry.depth {
            store_new_entry();
            return;
        }

        let existing_entry_score = entry.score as i16
            + (self.generation.load(Ordering::Relaxed) - entry.generation) as i16 * AGE_MULTIPLIER;

        if existing_entry_score <= score as i16 {
            store_new_entry();
        }
    }
}

// use crate::chess::consts::MAX_MOVES;
// use crate::chess::move_ply::MovePly;
// use crate::engine::types::tt_flag::TTFlag;
// use portable_atomic::{AtomicI16, AtomicU8, AtomicU16, AtomicU32, AtomicU128, Ordering};

// const ZOBRIST_SHIFT: u8 = 64;
// const SCORE_SHIFT: u8 = 48;
// const MOVE_SHIFT: u8 = 32;
// const EVAL_SHIFT: u8 = 16;
// const DEPTH_SHIFT: u8 = 8;
// const FLAG_SHIFT: u8 = 6;

// #[derive(Copy, Clone)]
// pub struct TTEntry {
//     pub zobrist: u64,
//     pub score: i16,
//     pub cur_move: MovePly,
//     pub eval: i16,
//     pub depth: u8,
//     pub tt_flag: TTFlag,
//     pub packed_data: u128,
// }

// impl From<u128> for TTEntry {
//     fn from(packed_data: u128) -> Self {
//         Self {
//             zobrist: (packed_data >> ZOBRIST_SHIFT) as u64,
//             score: ((packed_data >> SCORE_SHIFT) & 0xFFFF) as i16,
//             cur_move: (((packed_data >> MOVE_SHIFT) & 0xFFFF) as u16).into(),
//             eval: ((packed_data >> EVAL_SHIFT) & 0xFFFF) as i16,
//             depth: ((packed_data >> DEPTH_SHIFT) & 0xFF) as u8,
//             tt_flag: (((packed_data >> FLAG_SHIFT) & 0b11) as u8).into(),
//             packed_data,
//         }
//     }
// }

// const AGE_MULTIPLIER: i16 = -1;
// const EXACT_BONUS: i16 = 8;
// const DEPTH_MULTIPLIER: i16 = 4;

// pub struct Transposition {
//     entries: Box<[AtomicU128]>,
//     num_entries: u64,
//     generation: AtomicU16,

//     entries_filled: AtomicU128,

//     pub best_move_score: AtomicI16,
//     pub best_move: AtomicU16,
// }

// impl Transposition {
//     pub fn new(mb_size: u16) -> Self {
//         let entry_size = 16;
//         let size_as_bytes = mb_size as u64 * 1024 * 1024;
//         let max_num_entries = size_as_bytes / entry_size;

//         let round_down_pow2 = 1 << (63 - max_num_entries.leading_zeros());

//         let default_entry = (i16::MIN as u16) as u128;

//         let entries: Vec<AtomicU128> = (0..round_down_pow2)
//             .map(|_| AtomicU128::new(default_entry))
//             .collect();

//         Self {
//             entries: entries.into(),
//             num_entries: round_down_pow2,
//             generation: AtomicU16::new(0),
//             best_move: AtomicU16::new(0),
//             best_move_score: AtomicI16::new(0),
//             entries_filled: AtomicU128::new(0),
//         }
//     }

//     pub fn age(&self) {
//         let new_generation = self.generation.load(Ordering::Relaxed) + 1;
//         self.generation.store(new_generation, Ordering::Relaxed);
//     }

//     pub fn hash_full(&self) -> u32 {
//         0
//     }

//     pub fn probe(&self, zobrist: u64) -> Option<TTEntry> {
//         let index = zobrist & (self.num_entries - 1);
//         let packed_data = self.entries[index as usize].load(Ordering::Relaxed);

//         let unpacked_zobrist = (packed_data >> ZOBRIST_SHIFT) as u64;

//         if unpacked_zobrist != zobrist {
//             return None;
//         }

//         Some(TTEntry::from(packed_data))
//     }

//     pub fn update(
//         &self,
//         zobrist: u64,
//         cur_move: MovePly,
//         eval: i16,
//         depth: u8,
//         tt_flag: TTFlag,
//         _is_pv: bool,
//     ) {
//         let mut score: i16 = depth as i16 * DEPTH_MULTIPLIER
//             + self.generation.load(Ordering::Relaxed) as i16 * AGE_MULTIPLIER;

//         match tt_flag {
//             TTFlag::Exact => score += EXACT_BONUS,
//             _ => {}
//         }

//         let existing_entry = self.probe(zobrist);

//         // if existing_entry.is_some() && existing_entry.unwrap().score > score {
//         //     return;
//         // }

//         let packed_data: u128 = ((zobrist as u128) << ZOBRIST_SHIFT)
//             | (((score as u16) as u128) << SCORE_SHIFT)
//             | ((cur_move.packed_data() as u128) << MOVE_SHIFT)
//             | (((eval as u16) as u128) << EVAL_SHIFT)
//             | ((depth as u128) << DEPTH_SHIFT)
//             | ((tt_flag as u128) << FLAG_SHIFT);

//         let index = zobrist & (self.num_entries - 1);
//         self.entries[index as usize].store(packed_data, Ordering::Relaxed);
//     }
// }
