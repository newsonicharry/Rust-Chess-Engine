use std::sync::atomic::{AtomicU128, AtomicU16, Ordering};
use crate::chess::consts::MAX_MOVES;
use crate::chess::move_ply::MovePly;
use crate::engine::types::tt_flag::TTFlag;

const ZOBRIST_SHIFT: u8 = 64;
const SCORE_SHIFT: u8 = 48;
const MOVE_SHIFT: u8 = 32;
const EVAL_SHIFT: u8 = 16;
const DEPTH_SHIFT: u8 = 8;
const FLAG_SHIFT: u8 = 6;


#[derive(Copy, Clone)]
    pub struct TTEntry{
    pub zobrist: u64,
    pub score: i16,
    pub cur_move: MovePly,
    pub eval: i16,
    pub depth: u8,
    pub tt_flag: TTFlag,
}

impl From<u128> for TTEntry {
    fn from(packed_data: u128) -> Self {

        Self{
            zobrist:  (packed_data >> ZOBRIST_SHIFT) as u64,
            score:    ((packed_data >> SCORE_SHIFT) & 0xFFFF) as i16,
            cur_move: (((packed_data >> MOVE_SHIFT) & 0xFFFF) as u16).into(),
            eval:     ((packed_data >> EVAL_SHIFT)  & 0xFFFF) as i16,
            depth:    ((packed_data >> DEPTH_SHIFT) & 0xFF)   as u8,
            tt_flag:  (((packed_data >> FLAG_SHIFT) & 0b11)   as u8).into(),
        }

    }
}




const AGE_MULTIPLIER: i16 = -1;
const EXACT_BONUS: i16 = 8;
const DEPTH_MULTIPLIER: i16 = 4;


pub struct Transposition {
    entries: Box<[AtomicU128]>,
    num_entries: u64,
    generation: u16,

    pub best_move: AtomicU16,
}

impl Transposition {
    pub fn new(mb_size: u16) -> Self{
        let entry_size = 16;
        let size_as_bytes = mb_size as u64 * 1024 * 1024;
        let max_num_entries = size_as_bytes / entry_size;

        let round_down_pow2 = 1 << (63 - max_num_entries.leading_zeros());
        let entries: Vec<AtomicU128> = (0..round_down_pow2).map(|_| AtomicU128::new(0)).collect();
        Self{
            entries: entries.into(),
            num_entries: round_down_pow2,
            generation: MAX_MOVES as u16,
            best_move: AtomicU16::new(0)
        }
    }

    pub fn probe(&self, zobrist: u64) -> Option<TTEntry>{
        let index = zobrist & (self.num_entries - 1);
        let packed_data = self.entries[index as usize].load(Ordering::Relaxed);

        let unpacked_zobrist = (packed_data >> ZOBRIST_SHIFT) as u64;

        if packed_data == 0 || unpacked_zobrist != zobrist {
            return None;
        }

        Some(TTEntry::from(packed_data))
    }

    pub fn update(&self, zobrist: u64, cur_move: MovePly, eval: i16, depth: u8, tt_flag: TTFlag ) {

        let mut score: i16 = depth as i16 * DEPTH_MULTIPLIER + self.generation as i16 * AGE_MULTIPLIER;

        match tt_flag {
            TTFlag::Exact => score += EXACT_BONUS,
            _ => {}
        }

        let existing_entry = self.probe(zobrist);
        if existing_entry.is_some() && existing_entry.unwrap().score > score {
            return;
        }

        let packed_data: u128 = ((zobrist as u128) << ZOBRIST_SHIFT)
                                | ((score as u128) << SCORE_SHIFT)
                                | ((cur_move.packed_data() as u128) << MOVE_SHIFT)
                                | ((eval as u128) << EVAL_SHIFT)
                                | ((depth as u128) << DEPTH_SHIFT)
                                | ((tt_flag as u128) << FLAG_SHIFT);

        let index = zobrist & (self.num_entries - 1);
        self.entries[index as usize].store(packed_data, Ordering::Relaxed);
    }



}