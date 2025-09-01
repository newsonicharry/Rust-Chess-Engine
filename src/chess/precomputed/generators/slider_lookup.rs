use std::collections::HashSet;
use crate::chess::consts::{NUM_SQUARES, NUM_DIAGONAL_ENTRIES, NUM_ORTHOGONAL_ENTRIES, ROOK_DIRECTIONS, BISHOP_DIRECTIONS};
use crate::chess::precomputed::generators::helpers::{create_dynamic_mask, NO_EDGE};
use crate::chess::types::square::Square;
use rand::Rng;
use crate::chess::types::file::File;
use crate::chess::types::rank::Rank;
use crate::general::bits;



enum SliderType{
    Orthogonal,
    Diagonal,
}

pub struct SliderLookup<const NUM_ENTRIES: usize> {
    pub flat_table: [u64; NUM_ENTRIES],
    pub offsets: [usize; NUM_SQUARES],

    pub shifts: [u8; NUM_SQUARES],
    pub magics: [u64; NUM_SQUARES],

    pub no_edge_masks: [u64; NUM_SQUARES],
}

impl<const NUM_ENTRIES: usize> SliderLookup<NUM_ENTRIES> {
    pub fn new() -> Self{
        let mut lookup = SliderLookup{
            flat_table: [0; NUM_ENTRIES],
            offsets: [0; NUM_SQUARES],

            shifts: [0; NUM_SQUARES],
            magics: [0; NUM_SQUARES],

            no_edge_masks: [0; NUM_SQUARES],

        };

        let slider_type = match NUM_ENTRIES {
            NUM_DIAGONAL_ENTRIES => SliderType::Diagonal,
            NUM_ORTHOGONAL_ENTRIES => SliderType::Orthogonal,
            _ => unreachable!()
        };

        let piece_direction = match slider_type {
            SliderType::Diagonal => BISHOP_DIRECTIONS,
            SliderType::Orthogonal => ROOK_DIRECTIONS,
        };


        for i in 0..NUM_SQUARES {
            let square = Square::from(i as u8);

            lookup.no_edge_masks[i] |= create_dynamic_mask::<NO_EDGE>(&piece_direction, square);

            let (magic, shift) = lookup.find_magic_and_shift(square);
            lookup.magics[i] = magic;
            lookup.shifts[i] = shift;

        }


        lookup.generate_move_lookup(slider_type);

        lookup

    }





    fn generate_move_lookup(&mut self, slider_type: SliderType){

        let mut last_offset = 0;

        for (piece_index, piece_move_mask) in self.no_edge_masks.iter().enumerate(){

            let square = Square::from(piece_index as u8);
            let blockers = self.generate_blockers(square);

            let num_blockers = bits::count(*piece_move_mask);
            let blocker_combinations = 1 << num_blockers;


            for blocker in blockers {
                let magic = self.magics[piece_index];
                let shift = self.shifts[piece_index];
                // let key = blocker.wrapping_mul(magic) >> shift;
                let key = unsafe { std::arch::x86_64::_pext_u64(blocker, self.no_edge_masks[piece_index]) };

                let valid_moves =  self.get_moves_from_blockers(square, &slider_type, blocker);

                self.flat_table[key as usize + last_offset] = valid_moves;
            }

            self.offsets[piece_index] = last_offset;
            last_offset += blocker_combinations;
        }
    }



    fn get_moves_from_blockers(&self, square: Square, slider_type: &SliderType, blockers: u64) -> u64{

        let directions = match slider_type {
            SliderType::Diagonal => BISHOP_DIRECTIONS,
            SliderType::Orthogonal => ROOK_DIRECTIONS,
        };

        let mut new_movement_mask: u64 = 0;

        for direction in directions{
            let x_dir = direction.0;
            let y_dir = direction.1;

            let mut x_pos = square.file() as i8;
            let mut y_pos = square.rank() as i8;

            let mut directional_move_mask: u64 = 0;

            loop{

                x_pos += x_dir;
                y_pos += y_dir;

                if  (x_pos > 7 || x_pos < 0) || (y_pos > 7 || y_pos < 0) {
                    break
                }

                let file = File::from(x_pos as u8);
                let rank = Rank::from(y_pos as u8);
                let new_square = Square::from((file, rank));

                directional_move_mask |= new_square.mask();

                let test_x = x_pos + x_dir;
                let test_y = y_pos + y_dir;
                let end_of_board =(test_x > 7 || test_x < 0) || (test_y > 7 || test_y < 0);

                if directional_move_mask & blockers != 0 || end_of_board {
                    new_movement_mask |= directional_move_mask;
                    break
                }

            }

        }

        new_movement_mask

    }


    pub fn generate_blockers(&self, square: Square) -> Box<[u64]> {
        let no_edge_mask = self.no_edge_masks[square as usize];
        let squares = bits::all_squares(no_edge_mask);

        let total_blocker_patters = 1 << squares.len();
        let mut all_blocker_patters = vec![0u64; total_blocker_patters];

        for pattern_index in 0..total_blocker_patters {
            for square_index in 0..squares.len(){
                let bit = (pattern_index >> square_index) & 1;

                let current_move = squares.get(square_index).unwrap();

                all_blocker_patters[pattern_index] |= (bit << *current_move as u64) as u64;
            }

        }

        all_blocker_patters.into_boxed_slice()

    }



    fn find_magic_and_shift(&self, square: Square) -> (u64, u8){

        let no_edge_mask= self.no_edge_masks[square as usize];

        let num_blockers = bits::count(no_edge_mask);

        let blockers = self.generate_blockers(square);

        let mut final_magic: u64 = 0;


        loop {
            let mut all_keys = HashSet::with_capacity(blockers.len());

            let magic: u64 = rand::thread_rng().r#gen::<u64>() & rand::thread_rng().r#gen::<u64>() & rand::thread_rng().r#gen::<u64>();

            for blocker in blockers.iter(){
                let new_key = blocker.wrapping_mul(magic) >> (64 - num_blockers);

                if !all_keys.contains(&new_key) {
                    all_keys.insert(new_key);
                }
                else {
                    break;
                }

            }

            if all_keys.len() == blockers.len() { final_magic = magic; break; }

        }

        (final_magic, 64 - num_blockers)


    }



}