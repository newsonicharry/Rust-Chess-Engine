// #![allow(incomplete_features)]

use crate::chess::board::Board;
use crate::chess::move_generator::{BLACK, GEN_ALL, WHITE};
use crate::chess::move_generator::{IS_LEAF, MoveGenerator, NOT_LEAF};
use crate::chess::move_list::MoveList;
use std::time::Instant;

struct PerftTT {
    entries: Box<[u128]>,
}

impl PerftTT {
    pub fn new(mb_size: usize) -> Self {
        let size_as_bytes = mb_size * 1024 * 1024;
        let num_entries = size_as_bytes / 16; // 16 bytes per entry 
        let entries: Box<[u128]> = vec![0; num_entries].into_boxed_slice();
        Self { entries }
    }

    fn zobrist_from_entry(entry: u128) -> u64 {
        (entry >> 64) as u64
    }

    fn num_nodes_from_entry(entry: u128) -> u64 {
        (entry as u64) >> 4
    }

    pub fn probe(&self, zobrist: u64, depth: u8) -> Option<u64> {
        let index = (zobrist & (self.entries.len() as u64 - 1)) as usize;
        let entry = self.entries[index];

        if Self::zobrist_from_entry(entry) == zobrist && (entry as u8 & 0b1111) == depth {
            return Some(Self::num_nodes_from_entry(entry));
        }

        None
    }

    pub fn update(&mut self, zobrist: u64, num_nodes: u64, depth: u8) {
        let index = (zobrist & (self.entries.len() as u64 - 1)) as usize;
        let entry = self.entries[index];

        if Self::zobrist_from_entry(entry) == 0 || depth >= (entry as u8 & 0b1111) {
            self.entries[index] = ((zobrist as u128) << 64)
                | ((num_nodes as u64) << 4) as u128
                | (depth & 0b1111) as u128;
        }
    }
}

pub const PERFT: u8 = 0;
pub const BULK_PERFT: u8 = 1;
pub const TT_PERFT: u8 = 2;

macro_rules! generate_depth {
    ($name:tt, $calls:tt, $color:ident) => {
        #[inline(always)]
        fn $name<const PERFT_TYPE: u8, const DISPLAY: bool>(
            board: *mut Board,
            mut num_nodes: u64,
        ) -> u64 {
            MoveGenerator::<GEN_ALL>::const_generate::<$color, NOT_LEAF>(
                board,
                &mut |piece_moves| {
                    for curr_move in piece_moves {
                        unsafe {
                            (*board).make_move::<false>(&curr_move);
                        }

                        let curr_nodes = $calls::<PERFT_TYPE, false>(board, 0);

                        if DISPLAY {
                            println!("{curr_move}: {curr_nodes}");
                        }

                        num_nodes += curr_nodes;

                        unsafe {
                            (*board).undo_move();
                        }
                    }
                },
            );

            num_nodes
        }
    };
}

macro_rules! generate_specific_depth {
    ($white_name:tt, $white_calls:tt, $black_name:tt, $black_calls:tt) => {
        generate_depth!($white_name, $white_calls, WHITE);
        generate_depth!($black_name, $black_calls, BLACK);
    };
}

#[inline(always)]
fn search_w_1<const PERFT_TYPE: u8, const DISPLAY: bool>(
    board: *mut Board,
    _num_nodes: u64,
) -> u64 {
    return MoveGenerator::<GEN_ALL>::const_generate::<WHITE, IS_LEAF>(board, &mut |_| {});
}

#[inline(always)]
fn search_b_1<const PERFT_TYPE: u8, const DISPLAY: bool>(
    board: *mut Board,
    _num_nodes: u64,
) -> u64 {
    return MoveGenerator::<GEN_ALL>::const_generate::<BLACK, IS_LEAF>(board, &mut |_| {});
}

generate_specific_depth!(search_w_2, search_b_1, search_b_2, search_w_1);
generate_specific_depth!(search_w_3, search_b_2, search_b_3, search_w_2);
generate_specific_depth!(search_w_4, search_b_3, search_b_4, search_w_3);
generate_specific_depth!(search_w_5, search_b_4, search_b_5, search_w_4);
generate_specific_depth!(search_w_6, search_b_5, search_b_6, search_w_5);
generate_specific_depth!(search_w_7, search_b_6, search_b_7, search_w_6);
generate_specific_depth!(search_w_8, search_b_7, search_b_8, search_w_7);
// depth_generator!(search_w_9, search_b_8, search_b_9, search_w_8);

pub fn perft<const PERFT_TYPE: u8>(board: &mut Board, depth: u8) -> u64 {
    let mut transposition = PerftTT::new(128);

    let mut start_pos_moves = MoveList::default();
    MoveGenerator::<GEN_ALL>::generate::<NOT_LEAF>(board, &mut |mut piece_moves| {
        start_pos_moves.add_piece_moves(&mut piece_moves);
    });

    let timer = Instant::now();

    let all_nodes = match (depth, board.side_to_move().is_white()) {
        (1, WHITE) => search_w_1::<BULK_PERFT, true>(board, 0),
        (1, BLACK) => search_b_1::<BULK_PERFT, true>(board, 0),

        (2, WHITE) => search_w_2::<BULK_PERFT, true>(board, 0),
        (2, BLACK) => search_b_2::<BULK_PERFT, true>(board, 0),

        (3, WHITE) => search_w_3::<BULK_PERFT, true>(board, 0),
        (3, BLACK) => search_b_3::<BULK_PERFT, true>(board, 0),

        (4, WHITE) => search_w_4::<BULK_PERFT, true>(board, 0),
        (4, BLACK) => search_b_4::<BULK_PERFT, true>(board, 0),

        (5, WHITE) => search_w_5::<BULK_PERFT, true>(board, 0),
        (5, BLACK) => search_b_5::<BULK_PERFT, true>(board, 0),

        (6, WHITE) => search_w_6::<BULK_PERFT, true>(board, 0),
        (6, BLACK) => search_b_6::<BULK_PERFT, true>(board, 0),

        (7, WHITE) => search_w_7::<BULK_PERFT, true>(board, 0),
        (7, BLACK) => search_b_7::<BULK_PERFT, true>(board, 0),

        (8, WHITE) => search_w_8::<BULK_PERFT, true>(board, 0),
        (8, BLACK) => search_b_8::<BULK_PERFT, true>(board, 0),
        _ => {
            println!("Unexpected depth");
            return 0;
        }
    };

    let nodes_per_second = all_nodes as f64 / (timer.elapsed().as_secs_f64());
    let elapsed = timer.elapsed().as_secs_f64();

    println!("\nNodes searched: {all_nodes}");
    println!("Nodes per second: {nodes_per_second:.0}");
    println!("Seconds elapsed: {elapsed:.3}\n");

    all_nodes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_perft() {
        let mut board = Board::default();

        board.new("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
        assert_eq!(perft::<TT_PERFT>(&mut board, 7), 3_195_901_860);

        board = Board::default();
        board.new("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - ");
        assert_eq!(perft::<TT_PERFT>(&mut board, 5), 193_690_690);

        board = Board::default();
        board.new("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1");
        assert_eq!(perft::<TT_PERFT>(&mut board, 8), 3_009_794_393);

        board = Board::default();
        board.new("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1");
        assert_eq!(perft::<TT_PERFT>(&mut board, 6), 706_045_033);

        board = Board::default();
        board.new("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8");
        assert_eq!(perft::<TT_PERFT>(&mut board, 5), 89_941_194);

        board = Board::default();
        board.new("r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10");
        assert_eq!(perft::<TT_PERFT>(&mut board, 6), 6_923_051_137);
    }
}
