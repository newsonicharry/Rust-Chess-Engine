use crate::chess::board::Board;
use crate::chess::move_generator::GEN_ALL;
use crate::chess::move_generator::MoveGenerator;
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

fn search<const PERFT_TYPE: u8>(
    board: &mut Board,
    depth: u8,
    mut num_nodes: u64,
    transposition: &mut PerftTT,
) -> u64 {
    if depth == 0 && PERFT_TYPE == PERFT {
        return 1;
    }

    if let Some(tt_nodes) = transposition.probe(board.zobrist(), depth)
        && PERFT_TYPE == TT_PERFT
    {
        return tt_nodes;
    }

    let mut move_list = MoveList::default();
    MoveGenerator::<GEN_ALL>::generate(board, &mut move_list);

    if depth == 1 && (PERFT_TYPE == BULK_PERFT || PERFT_TYPE == TT_PERFT) {
        return move_list.move_count() as u64;
    }

    for cur_move in move_list.iter() {
        board.make_move(cur_move);
        let search_nodes = search::<PERFT_TYPE>(board, depth - 1, 0, transposition);

        if PERFT_TYPE == TT_PERFT {
            transposition.update(board.zobrist(), search_nodes, depth - 1);
        }

        num_nodes += search_nodes;

        board.undo_move();
    }

    num_nodes
}

pub fn perft<const PERFT_TYPE: u8>(board: &mut Board, depth: u8) -> u64 {
    let mut transposition = PerftTT::new(128);

    let mut start_pos_moves = MoveList::default();
    MoveGenerator::<GEN_ALL>::generate(board, &mut start_pos_moves);

    let timer = Instant::now();

    let mut all_nodes = 0;

    if depth == 1 {
        all_nodes = search::<PERFT_TYPE>(board, depth, 0, &mut transposition)
    } else {
        for cur_move in start_pos_moves.iter() {
            board.make_move(cur_move);
            let num_nodes = search::<PERFT_TYPE>(board, depth - 1, 0, &mut transposition);
            all_nodes += num_nodes;
            board.undo_move();
            println!("{cur_move}: {num_nodes}");
        }
    }

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
