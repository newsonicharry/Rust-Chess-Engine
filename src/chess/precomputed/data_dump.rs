use crate::chess::consts::{NUM_ORTHOGONAL_ENTRIES, NUM_DIAGONAL_ENTRIES};
use crate::chess::precomputed::generators::slider_lookup::SliderLookup;
use crate::chess::precomputed::generators::inbetween::InBetween;
use crate::chess::precomputed::generators::movement_mask::MovementMasks;
use std::fs;
use std::slice;
use crate::chess::precomputed::generators::zobrist::Zobrist;

pub fn dump_bins(){
    let orthogonal_lookup = SliderLookup::<NUM_ORTHOGONAL_ENTRIES>::new();
    let diagonal_lookup = SliderLookup::<NUM_DIAGONAL_ENTRIES>::new();
    
    let movement_mask = MovementMasks::new();
    let in_between = InBetween::new();
    let zobrist = Zobrist::new();

    dump_struct("src/chess/precomputed/bins/orthogonal_lookup.bin", &orthogonal_lookup).unwrap();
    dump_struct("src/chess/precomputed/bins/diagonal_lookup.bin", &diagonal_lookup).unwrap();
    // 
    dump_struct("src/chess/precomputed/bins/movement_masks.bin", &movement_mask).unwrap();
    dump_struct("src/chess/precomputed/bins/in_between.bin", &in_between).unwrap();
    dump_struct("src/chess/precomputed/bins/zobrist.bin", &zobrist).unwrap();

}

fn dump_struct<T>(path: &str, save_struct: &T) -> std::io::Result<()> {
    let pointer = save_struct as *const T as *const u8;

    let bytes: &[u8] = unsafe { slice::from_raw_parts(pointer, size_of::<T>()) };

    fs::write(path, bytes)?;

    Ok(())
}