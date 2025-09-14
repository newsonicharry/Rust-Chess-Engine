use crate::chess::board::Board;
use crate::chess::move_generator::MoveGenerator;
use crate::chess::move_generator::{GEN_ALL, GEN_TACTICS};
use crate::chess::move_list::MoveList;
use crate::chess::move_ply::MovePly;
use crate::chess::types::color::Color::White;
use crate::chess::consts::PIECE_VALUES;
use crate::engine::arbiter::Arbiter;
use crate::engine::eval::nnue::NNUE;
use crate::engine::search_limits::SearchLimits;
use crate::engine::thread::ThreadData;
use crate::engine::transposition::{TTEntry, Transposition};
use crate::engine::types::match_result::MatchResult;
use crate::engine::types::tt_flag::TTFlag;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::thread;
use crate::chess::types::piece::BasePiece::{King, Pawn};
use crate::engine::see::see;

const INFINITY: i16 = 30000;

fn search(
    board: &mut Board,
    ply_searched: u8,
    mut depth: u8,
    mut alpha: i16,
    beta: i16,
    thread_data: &mut ThreadData,
    tt: &Transposition,
    nnue: &mut NNUE,
    limits: &SearchLimits,
) -> i16{

    if limits.is_hard_stop() { return 0 }

    let tt_entry = tt.probe(board.zobrist());
    if let Some(entry) = tt_entry && ply_searched > 0 {
        if entry.depth >= depth{
            match entry.tt_flag{
                TTFlag::Exact => {return entry.eval},
                TTFlag::Upper => if entry.eval <= alpha {return entry.eval},
                TTFlag::Lower => if entry.eval >= beta {return entry.eval},
            }
        }
    }

    let mut move_list = MoveList::default();
    MoveGenerator::<GEN_ALL>::generate(board, &mut move_list);

    let match_result = Arbiter::arbitrate(board, &move_list);
    match match_result {
        MatchResult::Draw => {return 0},
        MatchResult::Loss => {return -INFINITY + ply_searched as i16},
        MatchResult::NoResult => {},
    }

    if board.in_check() { depth += 1; }

    if depth == 0{
        return quiescence_search(board, ply_searched+1, 8, alpha, beta, nnue, limits)
    }


    let pv_node = alpha != beta - 1;

    if !pv_node && depth > 6 && !board.in_check() && !in_zugzwang(board){

        board.make_null_move();
        let value = -search(board, ply_searched+1, depth-4, -beta, -(beta - 1), thread_data, tt, nnue, limits);
        if limits.is_hard_stop() { return 0 }
        board.undo_null_move();

        if value >= beta{
            return beta;
        }
    }

    let static_eval;
    if tt_entry.is_some() {
        static_eval = tt_entry.unwrap().eval;
    }else {
        static_eval = nnue.evaluate(board.side_to_move());
    }

    if static_eval >= (beta + 80 * depth as i16) {
        return beta;
    }

    order_moves(board, &mut move_list, &tt_entry, &thread_data, ply_searched);

    let mut node_type = TTFlag::Upper;
    let mut best_move = move_list.move_at( 0);


    for (i, cur_move) in move_list.iter().enumerate(){
        nnue.make_move(cur_move, board);
        board.make_move(cur_move);

        let mut eval;

        if i >= 3 && depth >= 3 {
            eval = -search(board, ply_searched+1, depth-2, -alpha - 1, -alpha, thread_data, tt, nnue, limits);

            if eval > alpha {
                eval = -search(board, ply_searched+1, depth-1, -beta, -alpha, thread_data, tt, nnue, limits);
            }
        }else {
            eval = -search(board, ply_searched+1, depth-1, -beta, -alpha, thread_data, tt, nnue, limits);
        }


        if limits.is_hard_stop() { return 0 }

        board.undo_move();
        nnue.undo_move();

        if eval >= beta {
            tt.update(board.zobrist(), *cur_move, beta, depth, TTFlag::Lower);
            thread_data.killers.update(*cur_move, ply_searched);
            thread_data.history_heuristics.update_history(*cur_move);
            return beta;
        }
        if eval > alpha {
            alpha = eval;
            node_type = TTFlag::Exact;
            best_move = *cur_move;
        }



    }

    if ply_searched == 0 {
        tt.best_move.store(best_move.packed_data(), Ordering::Relaxed);
        tt.best_move_score.store(alpha, Ordering::Relaxed);
    }

    tt.update(board.zobrist(), best_move, alpha, depth, node_type);

    alpha
}

fn in_zugzwang(board: &Board) -> bool {
    let king_pawn_occupancy = board.bitboard_combined(Pawn) | board.bitboard_combined(King);
    if board.occupancy() == king_pawn_occupancy {
        return true;
    }

    false
}


fn quiescence_search(
    board: &mut Board,
    ply_searched: u8,
    depth: u8,
    mut alpha: i16,
    beta: i16,
    nnue: &mut NNUE,
    limits: &SearchLimits,
) -> i16{
    if limits.is_hard_stop() { return 0 }


    let eval = nnue.evaluate(board.side_to_move());

    if eval >= beta { return beta }
    if alpha < eval { alpha = eval }

    if depth == 0 {
        return alpha;
    }

    let mut move_list = MoveList::default();
    MoveGenerator::<GEN_TACTICS>::generate(board, &mut move_list);

    if Arbiter::is_draw(board) {
        return 0
    }

    for cur_move in move_list.iter(){
        // if see(cur_move.from(), cur_move.to(), board).is_negative() {
        //     continue;
        // }

        nnue.make_move(cur_move, board);
        board.make_move(cur_move);

        let eval = -quiescence_search(board, ply_searched+1, depth-1, -beta, -alpha, nnue, limits);
        if limits.is_hard_stop() { return 0 }

        board.undo_move();
        nnue.undo_move();


        if eval < alpha {
            alpha = eval;
        }

    }


    alpha
}

pub fn iterative_deepening(
    board: &mut Board,
    tt: &Transposition,
    nnue: &mut NNUE,
    search_limits: &SearchLimits
    ) -> MovePly{

    let mut thread_data = ThreadData::default();

    for cur_depth in 1.. {
    
        search(board, 0, cur_depth, -INFINITY, INFINITY, &mut thread_data, tt, nnue, search_limits);
        if search_limits.is_hard_stop() { break }
    
        println!("info depth {} score cp {} time {}", cur_depth, tt.best_move_score.load(Ordering::Relaxed), search_limits.ms_elapsed());
    
        if search_limits.is_soft_stop() { break }
    }

    let best_move: MovePly = tt.best_move.load(Ordering::Relaxed).into();
    best_move

}



fn order_moves(board: &Board, move_list: &mut MoveList, prev_best_move: &Option<TTEntry>, thread_data: &ThreadData, ply_searched: u8) {

    let mut move_values: [i16; 256] = [0; 256];


    for (i, cur_move) in move_list.iter().enumerate(){
        let from = cur_move.from();
        let to = cur_move.to();
        let flag = cur_move.flag();

        let piece = board.piece_at(from);
        let capture = board.piece_at(to);


        if let Some(entry) = prev_best_move {
            if entry.cur_move == *cur_move {
                move_values[i] = INFINITY;
                continue;
            }
        }

        if thread_data.killers.contains(ply_searched, *cur_move) {
            move_values[i] += 5000;
        }

        // will replace later with a proper SEE implementation
        if capture.is_piece() {
            move_values[i] += 5 * PIECE_VALUES[capture as usize] - PIECE_VALUES[piece as usize];
        }

        if flag.is_promotion(){
            move_values[i] +=  PIECE_VALUES[flag.promotion_piece(White) as usize];
        }

        if flag.is_castles() {
            move_values[i] += 1000;
        }

        move_values[i] += thread_data.history_heuristics.get_history(*cur_move) as i16;

    }

    move_list.order_moves(&move_values);

}

pub fn search_start(
    num_threads: usize,
    mut board: Board,
    tt: &Arc<Transposition>,
    search_limits: &SearchLimits){

    let mut nnue = NNUE::default();
    nnue.new(&mut board);

    let mut thread_board = board.clone();

    let tt_new = Arc::clone(&tt);

    let best_move = iterative_deepening(&mut thread_board, &tt_new, &mut nnue, &search_limits);
    println!("bestmove {}\n",  best_move);

    // let limits = search_limits.clone();
    // let mut handles = Vec::new();
    // for _ in 0..8 {
    //     let tt_new = Arc::clone(&tt);
    //
    //     let handle = thread::Builder::new().stack_size(32 * 1024 * 1024).name("eiei".to_string()).spawn(move || {
    //         let mut thread_board = board.clone();
    //         let mut thread_nnue = nnue.clone();
    //         iterative_deepening(&mut thread_board, &tt_new, &mut thread_nnue, &limits);
    //     });
    //
    //     handles.push(handle.unwrap());
    // }
    //
    //
    // for handle in handles {
    //
    //     handle.join().unwrap();
    // }
}