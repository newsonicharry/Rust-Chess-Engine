use crate::chess::board::Board;
use crate::chess::move_generator::MoveGenerator;
use crate::chess::move_list::MoveList;
use crate::engine::arbiter::Arbiter;
use crate::engine::eval::nnue::NNUE;
use crate::engine::search_limits::SearchLimits;
use crate::engine::transposition::{TTEntry, Transposition};
use crate::engine::types::match_result::MatchResult;
use crate::engine::types::tt_flag::TTFlag;
use crate::chess::move_generator::{GEN_ALL, GEN_TACTICS};
use std::thread::Thread;

const INFINITY: i16 = 30000;

pub fn search(
    board: &mut Board,
    ply_searched: u8,
    depth: u8,
    mut alpha: i16,
    beta: i16,
    thread_data: &mut Thread,
    tt: &mut Transposition,
    nnue: &mut NNUE,
    limits: &SearchLimits,
) -> i16{

    let mut move_list = MoveList::default();
    MoveGenerator::<GEN_ALL>::generate(board, &mut move_list);

    let match_result = Arbiter::arbitrate(board, &move_list);
    match match_result {
        MatchResult::Draw => {return 0},
        MatchResult::Loss => {return -INFINITY + ply_searched as i16},
        MatchResult::NoResult => {},
    }

    let tt_entry = tt.probe(board.zobrist());
    if let Some(entry) = tt_entry {
        if entry.depth >= depth{
            match entry.tt_flag{
                TTFlag::Exact => {return entry.eval},
                TTFlag::Upper => if entry.eval <= alpha {return entry.eval},
                TTFlag::Lower => if entry.eval >= beta {return entry.eval},
            }
        }
    }

    if depth == 0{
        return quiescence_search()
    }

    order_moves(board, &move_list, &tt_entry);

    let mut node_type = TTFlag::Upper;
    let mut best_move = &move_list.move_at(0);
    let mut best_eval = -INFINITY;
    
    for cur_move in move_list.iter(){
        nnue.make_move(cur_move, board);
        board.make_move(cur_move);
        
        let eval = -search(board, ply_searched+1, depth-1, -beta, -alpha, thread_data, tt, nnue, limits);
        
        board.undo_move();
        nnue.undo_move();
        
        if eval >= beta { 
            tt.update(board.zobrist(), *cur_move, eval, depth, TTFlag::Lower)
        }
        if eval < alpha { 
            alpha = eval;
            node_type = TTFlag::Exact;
        }
        
        if eval > best_eval { 
            best_move = cur_move;
            best_eval = eval;
        }
        
    }

    tt.update(board.zobrist(), *best_move, best_eval, depth, node_type);
    
    alpha
}

pub fn quiescence_search() -> i16{
    0
}

pub fn order_moves(board: &Board, move_list: &MoveList, prev_best_move: &Option<TTEntry>){

}

pub fn iterative_deepening(){
    
}