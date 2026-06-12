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

const NO_SCORING: bool = false;
const ENTRIES_PER_BUCKETS: usize = 2;
const ENTRY_SIZE: usize = std::mem::size_of::<TTEntry>();

pub struct Transposition {
    bucket_entries: Box<[TTBucket]>,
    num_entries: u64,
    pub generation: AtomicU8,

    entries_filled: AtomicU32,
    pub best_move_score: AtomicI16,
    pub best_move: AtomicU16,
}

impl Transposition {
    pub fn new(mb_size: u16) -> Self {
        let bucket_size_bytes = ENTRY_SIZE as u64 * ENTRIES_PER_BUCKETS as u64;
        let size_as_bytes = mb_size as u64 * 1024 * 1024;
        let max_num_buckets = size_as_bytes / bucket_size_bytes;

        let round_down_pow2 = 1 << (63 - max_num_buckets.leading_zeros());

        let entries: Vec<TTBucket> = (0..round_down_pow2).map(|_| TTBucket::default()).collect();

        Self {
            bucket_entries: entries.into(),
            num_entries: round_down_pow2 * ENTRIES_PER_BUCKETS as u64,
            generation: AtomicU8::new(0),
            entries_filled: AtomicU32::new(0),
            best_move: AtomicU16::new(0),
            best_move_score: AtomicI16::new(0),
        }
    }

    pub fn hash_full(&self) -> u32 {
        self.entries_filled.load(Ordering::Relaxed) * 1000 / self.num_entries as u32
    }

    #[allow(dead_code)]
    pub fn age(&self) {
        // dont want an overflow
        if self.generation.load(Ordering::Relaxed) == u8::MAX {
            return;
        }
        self.generation.add(1, Ordering::Relaxed);
    }

    pub fn probe(&self, zobrist: u64) -> Option<TTEntry> {
        let index = zobrist & (self.bucket_entries.len() as u64 - 1);
        self.bucket_entries[index as usize].get(zobrist)
    }

    pub fn update(
        &self,
        zobrist: u64,
        cur_move: MovePly,
        eval: i16,
        depth: u8,
        tt_flag: TTFlag,
        is_pv: bool,
    ) {
        let mut score = depth as i16 * DEPTH_MULTIPLIER + 0 as i16 * EXACT_BONUS;

        match tt_flag {
            TTFlag::Exact => score += EXACT_BONUS,
            _ => {}
        }

        let score = score.clamp(0, 255) as u8;

        let new_entry = TTEntry {
            zobrist,
            cur_move,
            eval,
            depth,
            score,
            generation: self.generation.load(Ordering::Relaxed),
            tt_flag,
        };

        let index = zobrist & (self.bucket_entries.len() as u64 - 1);
        let updated_empty_entry = self.bucket_entries[index as usize]
            .update(&new_entry, self.generation.load(Ordering::Relaxed));

        // update the hash filled if its an empty entry
        if updated_empty_entry {
            self.entries_filled.add(1, Ordering::Relaxed);
        }
    }
}

// zobrist: 64
// curmove: 16
// eval   : 16
// depth  : 8 (could be 6)
// flag   : 2
// score  : 8
// gen    : 8

// total  : 122
// remain : 6

struct TTBucket {
    bucket: Box<[AtomicU128]>,
}

impl Default for TTBucket {
    fn default() -> Self {
        let buckets: Vec<AtomicU128> = (0..ENTRIES_PER_BUCKETS)
            .map(|_| AtomicU128::new(0))
            .collect();
        Self {
            bucket: buckets.into_boxed_slice(),
        }
    }
}

impl TTBucket {
    pub fn get(&self, zobrist: u64) -> Option<TTEntry> {
        for entry_data in self.bucket.iter() {
            let tt_entry: TTEntry = unsafe { mem::transmute(entry_data.load(Ordering::Relaxed)) };
            if tt_entry.zobrist == zobrist {
                return Some(tt_entry);
            }
        }

        return None;
    }

    // returns a boolean based on if it replaced an empty entry
    pub fn update(&self, new_entry: &TTEntry, curr_generation: u8) -> bool {
        if NO_SCORING {
            self.bucket[0].store(unsafe { mem::transmute(*new_entry) }, Ordering::Relaxed);
            return false;
        }

        let mut lowest_score: i16 = i16::MAX;
        let mut lowest_score_index = 0;

        for (index, entry_data) in self.bucket.iter().enumerate() {
            let entry: TTEntry = entry_data.load(Ordering::Relaxed).into();

            // if its an empty entry its safe to replace
            if entry.zobrist == 0 {
                self.bucket[index].store(unsafe { mem::transmute(*new_entry) }, Ordering::Relaxed);
                return true;
            }

            if entry.zobrist == new_entry.zobrist && entry.depth < new_entry.depth {
                self.bucket[index].store(unsafe { mem::transmute(*new_entry) }, Ordering::Relaxed);
                return false;
            }

            let entry_score =
                entry.score as i16 + (curr_generation - entry.generation) as i16 * AGE_MULTIPLIER;

            if entry_score < lowest_score {
                lowest_score = entry_score;
                lowest_score_index = index;
            }
        }

        if lowest_score < new_entry.score as i16 {
            self.bucket[lowest_score_index]
                .store(unsafe { mem::transmute(*new_entry) }, Ordering::Relaxed);
            return false;
        }

        false
    }
}
