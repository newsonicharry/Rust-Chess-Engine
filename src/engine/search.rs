use crate::chess::board::Board;
use crate::chess::move_generator::MoveGenerator;
use crate::chess::move_generator::{GEN_ALL, GEN_TACTICS};
use crate::chess::move_list::MoveList;
use crate::engine::arbiter::Arbiter;
use crate::engine::eval::nnue::NNUE;
use crate::engine::search_limits::SearchLimits;
use crate::engine::transposition::{TTEntry, Transposition};
use crate::engine::types::match_result::MatchResult;
use crate::engine::types::tt_flag::TTFlag;
use std::sync::Arc;
use std::thread;
use std::time::Instant;

const INFINITY: i16 = 30000;

fn search(
    board: &mut Board,
    ply_searched: u8,
    depth: u8,
    mut alpha: i16,
    beta: i16,
    // thread_data: &mut Thread,
    tt: &Transposition,
    nnue: &mut NNUE,
    limits: &SearchLimits,
) -> i16{

    if limits.hard_stop() { return 0 }

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

    if depth <= 0{
        return quiescence_search(board, ply_searched+1, 8, alpha, beta, nnue, limits)
    }

    order_moves(board, &move_list, &tt_entry);

    let mut node_type = TTFlag::Upper;
    let mut best_move = &move_list.move_at( 0);
    let mut best_eval = -INFINITY;


    for cur_move in move_list.iter(){
        nnue.make_move(cur_move, board);
        board.make_move(cur_move);

        if ply_searched == 0 {
            println!("cur move {cur_move}");
        }

        let eval = -search(board, ply_searched+1, depth-1, -beta, -alpha, tt, nnue, limits);

        if limits.hard_stop() { return 0 }

        board.undo_move();
        nnue.undo_move();

        if eval >= beta {
            tt.update(board.zobrist(), *cur_move, eval, depth, TTFlag::Lower);
            return beta;
        }
        if eval > alpha {
            alpha = eval;
            node_type = TTFlag::Exact;
            best_move = cur_move;
        }



    }

    if ply_searched == 0 {
        // tt.best_move = best_move.clone();
    }

    tt.update(board.zobrist(), *best_move, best_eval, depth, node_type);

    alpha
}

fn quiescence_search(
    board: &mut Board,
    ply_searched: u8,
    depth: u8,
    mut alpha: i16,
    beta: i16,
    // thread_data: &mut Thread,
    nnue: &mut NNUE,
    limits: &SearchLimits,
) -> i16{
    if limits.hard_stop() { return 0 }


    let eval = nnue.evaluate(board.side_to_move());

    if eval >= beta { return beta }
    if alpha < eval { alpha = eval }

    if depth == 0 {
        return alpha;
    }

    let mut move_list = MoveList::default();
    MoveGenerator::<GEN_TACTICS>::generate(board, &mut move_list);


    for cur_move in move_list.iter(){
        nnue.make_move(cur_move, board);
        board.make_move(cur_move);

        let eval = -quiescence_search(board, ply_searched+1, depth-1, -beta, -alpha, nnue, limits);
        if limits.hard_stop() { return 0 }

        board.undo_move();
        nnue.undo_move();


        if eval < alpha {
            alpha = eval;
        }

    }


    alpha
}

pub(crate) fn iterative_deepening(
    board: &mut Board,
    tt: &Transposition,
    nnue: &mut NNUE,
    search_limits: &SearchLimits
    ){
    
    for cur_depth in 1.. {
    
        search(board, 0, cur_depth, -INFINITY, INFINITY, tt, nnue, search_limits);
        if search_limits.hard_stop() { break }
    
        println!("info depth {}", cur_depth);
    
        if search_limits.is_soft_stop() { break }
    }

    println!("bestmove {}", tt.best_move)

}

pub fn order_moves(board: &Board, move_list: &MoveList, prev_best_move: &Option<TTEntry>){

}

pub fn search_start(
    num_threads: usize,
    board: Board,
    tt: Arc<Transposition>,
    nnue: NNUE,){
    
    let mut handles = Vec::new();
    for _ in 0..num_threads {


        let tt_new = Arc::clone(&tt);


        let handle = thread::Builder::new().stack_size(8 * 1024 * 1024).spawn(move || {
            let mut thread_board = board.clone();
            let mut thread_nnue = nnue.clone();
            search(&mut thread_board, 0, 7, -INFINITY, INFINITY, &tt_new, &mut thread_nnue, &SearchLimits::new(1000000000000, 1000000000000))
        });

        handles.push(handle.unwrap());
    }


    for handle in handles {

        handle.join().unwrap();
    }
}