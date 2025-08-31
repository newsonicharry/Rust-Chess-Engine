use crate::chess::precomputed::generators::inbetween::InBetween;
use crate::chess::precomputed::generators::movement_mask::MovementMasks;
use crate::chess::precomputed::generators::slider_lookup::SliderLookup;
use crate::chess::consts::{NUM_ORTHOGONAL_ENTRIES, NUM_DIAGONAL_ENTRIES, USE_BMI2};
use crate::chess::precomputed::generators::zobrist::Zobrist;
use crate::chess::types::square::Square;

pub static MOVEMENT_MASKS: MovementMasks = unsafe { std::mem::transmute(*include_bytes!("./bins/movement_masks.bin")) };
pub static IN_BETWEEN: InBetween = unsafe { std::mem::transmute(*include_bytes!("./bins/in_between.bin")) };
pub static ZOBRIST: Zobrist = unsafe { std::mem::transmute(*include_bytes!("./bins/zobrist.bin")) };

static DIAGONAL_LOOKUP: SliderLookup<NUM_DIAGONAL_ENTRIES> = unsafe { std::mem::transmute(*include_bytes!("./bins/diagonal_lookup.bin")) };
static ORTHOGONAL_LOOKUP: SliderLookup<NUM_ORTHOGONAL_ENTRIES> = unsafe { std::mem::transmute(*include_bytes!("./bins/orthogonal_lookup.bin")) };


pub fn rook_lookup(square: Square, occupied: u64) -> u64{
    let magic = ORTHOGONAL_LOOKUP.magics[square as usize];
    let shift = ORTHOGONAL_LOOKUP.shifts[square as usize];
    let blockers = ORTHOGONAL_LOOKUP.no_edge_masks[square as usize]; & occupied;

    let key = blockers.wrapping_mul(magic) >> shift;
    ORTHOGONAL_LOOKUP.flat_table[key as usize]
}

pub fn bishop_lookup(square: Square, occupied: u64) -> u64{
    let magic = DIAGONAL_LOOKUP.magics[square as usize];
    let shift = DIAGONAL_LOOKUP.shifts[square as usize];
    let blockers = DIAGONAL_LOOKUP.no_edge_masks[square as usize]; & occupied;

    let key = blockers.wrapping_mul(magic) >> shift;
    DIAGONAL_LOOKUP.flat_table[key as usize]
}

pub fn queen_lookup(square: Square, occupied: u64) -> u64{
    rook_lookup(square, occupied) | bishop_lookup(square, occupied)
}