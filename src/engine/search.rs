use crate::chess::board::Board;
use crate::chess::consts::PIECE_VALUES;
use crate::chess::move_generator::MoveGenerator;
use crate::chess::move_generator::{GEN_ALL, GEN_TACTICS};
use crate::chess::move_list::MoveList;
use crate::chess::move_ply::MovePly;
use crate::chess::types::color::Color::White;
use crate::chess::types::piece::BasePiece::{King, Pawn};
use crate::engine::arbiter::Arbiter;
use crate::engine::eval::nnue::NNUE;
use crate::engine::search_limits::SearchLimits;
use crate::engine::search_funcs::{see, move_is_quiet};
use crate::engine::thread::ThreadData;
use crate::engine::transposition::{TTEntry, Transposition};
use crate::engine::types::match_result::MatchResult;
use crate::engine::types::tt_flag::TTFlag;
use std::sync::atomic::Ordering;
use std::sync::Arc;

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

    thread_data.nodes += 1;

    let tt_entry = tt.probe(board.zobrist());
    if let Some(entry) = tt_entry {
        if entry.depth >= depth{
            match entry.tt_flag{
                TTFlag::Exact => {
                    if ply_searched == 0 {
                        tt.best_move.store(entry.cur_move.packed_data(), Ordering::Relaxed);
                        tt.best_move_score.store(entry.eval, Ordering::Relaxed);
                    }

                    return entry.eval
                },
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
        return quiescence_search(board, ply_searched+1, 8, alpha, beta, thread_data, tt, nnue, limits)
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

    let static_eval = tt_entry.map(|e| e.eval).unwrap_or_else(|| nnue.evaluate(board.side_to_move()));

    if static_eval >= (beta + 80 * depth as i16) {
        return beta;
    }

    if depth > 4 && tt_entry.is_none() {
        depth -= 1;
    }

    order_moves(board, &mut move_list, &tt_entry, &thread_data, ply_searched);

    let mut node_type = TTFlag::Upper;
    let mut best_eval = -INFINITY;
    let mut best_move = move_list.move_at( 0);


    for (i, cur_move) in move_list.iter().enumerate(){
        // if depth == 1 && static_eval + 200 <= alpha {
        //     continue;
        // }


        nnue.make_move(cur_move, board);
        board.make_move(cur_move);

        let mut eval;

        if i >= 3 && depth >= 3{
            eval = -search(board, ply_searched+1, depth-2, -alpha - 1, -alpha, thread_data, tt, nnue, limits);
            if limits.is_hard_stop() { return 0 }

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
            thread_data.history_heuristics.update_history(*cur_move, board.side_to_move());
            thread_data.counter_move_heuristics.update_counter_move(*cur_move, best_move, board.side_to_move());
            return beta;
        }
        if eval > alpha {
            alpha = eval;
            node_type = TTFlag::Exact;
            best_move = *cur_move;
        }

        if eval > best_eval {
            best_eval = eval;
            best_move = *cur_move;
        }



    }
    tt.update(board.zobrist(), best_move, alpha, depth, node_type);

    if ply_searched == 0 {
        tt.best_move.store(best_move.packed_data(), Ordering::Relaxed);
        tt.best_move_score.store(alpha, Ordering::Relaxed);
    }


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
    thread_data: &mut ThreadData,
    tt: &Transposition,
    nnue: &mut NNUE,
    limits: &SearchLimits,
) -> i16{
    if limits.is_hard_stop() { return 0 }

    thread_data.nodes += 1;


    let tt_entry = tt.probe(board.zobrist());
    if let Some(entry) = tt_entry {
        if entry.depth >= depth{
            match entry.tt_flag{
                TTFlag::Exact => { return entry.eval },
                TTFlag::Upper => if entry.eval <= alpha {return entry.eval},
                TTFlag::Lower => if entry.eval >= beta {return entry.eval},
            }
        }
    }

    let eval = nnue.evaluate(board.side_to_move());

    if eval >= beta { return beta }
    if alpha < eval { alpha = eval }

    if depth == 0 {
        return alpha;
    }

    let mut all_move_list = MoveList::default();
    MoveGenerator::<GEN_ALL>::generate(board, &mut all_move_list);

    let match_result = Arbiter::arbitrate(board, &all_move_list);
    match match_result {
        MatchResult::Draw => {return 0},
        MatchResult::Loss => {return -INFINITY + ply_searched as i16},
        MatchResult::NoResult => {},
    }

    let mut quiet_move_list = MoveList::default();
    MoveGenerator::<GEN_TACTICS>::generate(board, &mut quiet_move_list);

    // if Arbiter::is_draw(board) {
    //     return 0;
    // }


    let mut node_type = TTFlag::Upper;
    let mut best_move = quiet_move_list.move_at( 0);

    for cur_move in quiet_move_list.iter(){
        // if see(cur_move.from(), cur_move.to(), board).is_negative() {
        //     continue;
        // }

        nnue.make_move(cur_move, board);
        board.make_move(cur_move);

        let eval = -quiescence_search(board, ply_searched+1, depth-1, -beta, -alpha, thread_data, tt, nnue, limits);
        if limits.is_hard_stop() { return 0 }

        board.undo_move();
        nnue.undo_move();

        if eval >= beta{
            tt.update(board.zobrist(), best_move, alpha, 0, TTFlag::Lower);
            return beta;
        }

        if eval > alpha {
            best_move = *cur_move;
            node_type = TTFlag::Exact;
            alpha = eval;
        }

    }

    tt.update(board.zobrist(), best_move, alpha, 0, node_type);

    alpha
}


fn pv_from_transposition(board: &Board, tt: &Transposition) -> String {
    let mut board_clone = board.clone();
    let mut pv_line: String = "".to_string();

    loop{
        if pv_line.chars().filter(|&cur_char| ' ' == cur_char).count() == 20 {
            break;
        }

        if let Some(entry) = tt.probe(board_clone.zobrist()){
            let best_move = entry.cur_move;

            let mut valid_moves = MoveList::default();
            MoveGenerator::<GEN_ALL>::generate(&mut board_clone, &mut valid_moves);

            if !valid_moves.contains_move(best_move) {
                break;
            }


            pv_line += &*(best_move.to_string().as_str().to_owned() + " ");
            board_clone.make_move(&best_move);
        }else {
            break
        }

    }

    pv_line

}

fn aspiration_windows(
    board: &mut Board,
    tt: &Transposition,
    nnue: &mut NNUE,
    search_limits: &SearchLimits,
    thread_data: &mut ThreadData,
    depth: u8,
    ){

    let mut alpha = -INFINITY;
    let mut beta = INFINITY;
    let mut delta = 25;

    if depth >= 5 {
        let current_eval = tt.best_move_score.load(Ordering::Relaxed);
        alpha = current_eval - delta;
        beta = current_eval + delta;
    }


    loop{
        let eval = search(board, 0, depth, alpha, beta, thread_data, tt, nnue, search_limits);

        if search_limits.is_hard_stop() { return; }

        if eval <= alpha {
            beta = (alpha + beta) / 2;
            alpha -= delta;
        } else if eval >= beta{
            beta += delta;
        }else {
            break;
        }

        delta += delta / 2;
        if delta >= 1000{
            alpha = -INFINITY;
            beta = INFINITY;
        }

    }

}
pub fn iterative_deepening(
    board: &mut Board,
    tt: &Transposition,
    nnue: &mut NNUE,
    search_limits: &SearchLimits
    ) -> MovePly{

    let mut thread_data = ThreadData::default();

    for cur_depth in 1.. {

        aspiration_windows(board, tt, nnue, search_limits, &mut thread_data, cur_depth);
        if search_limits.is_hard_stop() { break }

        let pv_line = pv_from_transposition(board, tt);
        let sel_depth = pv_line.split(" ").collect::<Vec<&str>>().len()-1;
        let time = search_limits.ms_elapsed();
        let eval = tt.best_move_score.load(Ordering::Relaxed);
        let nodes = thread_data.nodes;
        let nps = (nodes as f64 / (time as f64 / 1000f64).max(0.0001f64)) as u128;
        println!("info depth {cur_depth} seldepth {sel_depth} score cp {eval} nodes {nodes} nps {nps} time {time} pv {pv_line}");
    
        if search_limits.is_soft_stop() { break }
    }

    let best_move: MovePly = tt.best_move.load(Ordering::Relaxed).into();
    best_move

}



fn order_moves(board: &Board, move_list: &mut MoveList, prev_best_move: &Option<TTEntry>, thread_data: &ThreadData, ply_searched: u8) {

    let mut move_values: [i16; 256] = [0; 256];

    let mut counter_move = MovePly::default();
    let last_played = board.last_move();

    if let Some(last_move) = last_played {
        counter_move = thread_data.counter_move_heuristics.get_counter_move(last_move, board.side_to_move());
    }

    for (i, cur_move) in move_list.iter().enumerate(){
        let flag = cur_move.flag();

        if let Some(entry) = prev_best_move {
            if entry.cur_move == *cur_move {
                move_values[i] = INFINITY;
                continue;
            }
        }

        if *cur_move == counter_move {
            move_values[i] += 300;
        }

        if thread_data.killers.contains(ply_searched, *cur_move) {
            move_values[i] += 5000;
        }

        move_values[i] += 5*(see(cur_move.from(), cur_move.to(), board).max(0));

        if flag.is_promotion(){
            move_values[i] += PIECE_VALUES[flag.promotion_piece(White) as usize]*10;
        }

        if flag.is_castles() {
            move_values[i] += 1000;
        }

        move_values[i] += thread_data.history_heuristics.get_history(*cur_move, board.side_to_move()) as i16;

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

    // let tt_new? = Arc::clone(&tt);

    let best_move = iterative_deepening(&mut thread_board, &tt, &mut nnue, &search_limits);
    println!("bestmove {}\n",  best_move);
    // tt.age();


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