use crate::chess::board::Board;
use crate::chess::consts;
use crate::chess::consts::PIECE_VALUES;
use crate::chess::move_generator::MoveGenerator;
use crate::chess::move_generator::{GEN_ALL, GEN_TACTICS, NOT_LEAF};
use crate::chess::move_list::MoveList;
use crate::chess::move_ply::MovePly;
use crate::chess::types::color::Color::White;
use crate::chess::types::piece::BasePiece::{King, Pawn};
use crate::engine::arbiter::Arbiter;
use crate::engine::eval::nnue::NNUE;
use crate::engine::heuristics::{CounterMoveHeuristics, History, KillerMoves};
use crate::engine::search_funcs::move_is_capture;
use crate::engine::search_limits::SearchLimits;
use crate::engine::transposition::{TTEntry, Transposition};
use crate::engine::types::match_result::MatchResult;
use crate::engine::types::tt_flag::TTFlag;
use crate::precomputed::accessor::LMR_REDUCTION;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::thread;

const INFINITY: i16 = 30000;

pub struct Searcher {
    board: Board,
    search_stack: [MovePly; consts::MAX_MOVES],
    counter_move_heuristics: CounterMoveHeuristics,
    killers: KillerMoves,
    history: History,
    nodes: u128,
    tt: Arc<Transposition>,
    nnue: NNUE,
    search_limits: SearchLimits,
}

const IS_ROOT: bool = true;
const NOT_ROOT: bool = false;

impl Searcher {
    pub fn new(
        transposition: &Arc<Transposition>,
        board: &Board,
        search_limits: &SearchLimits,
    ) -> Self {
        Self {
            board: board.clone(),
            search_stack: [MovePly::default(); consts::MAX_MOVES],
            counter_move_heuristics: CounterMoveHeuristics::default(),
            killers: KillerMoves::default(),
            history: History::default(),
            nodes: 0,
            tt: Arc::clone(transposition),
            nnue: NNUE::new(board.clone()),
            search_limits: search_limits.clone(),
        }
    }

    fn adjust_mate_distance(mut eval: i16, ply: u8) -> i16 {
        if eval > 29000 {
            eval -= ply as i16;
        } else if eval < -29000 {
            eval += ply as i16;
        }

        eval
    }

    fn search<const ROOT: bool>(
        &mut self,
        ply: u8,
        mut depth: u8,
        mut alpha: i16,
        beta: i16,
    ) -> i16 {
        if self.search_limits.is_hard_stop() {
            return 0;
        }

        // let mating_value = INFINITY - ply as i16;
        // if mating_value < beta {
        //     beta = mating_value;
        //     if alpha >= beta {
        //         return beta;
        //     }
        // }

        // let mated_value = -INFINITY + ply as i16;
        // if mated_value > alpha {
        //     alpha = mated_value;
        //     if alpha >= beta {
        //         return alpha;
        //     }
        // }

        if self.board.in_check() {
            depth += 1;
        }

        let mut move_list = MoveList::default();
        MoveGenerator::<GEN_ALL>::generate::<NOT_LEAF>(&mut self.board, &mut |mut piece_moves| {
            move_list.add_piece_moves(&mut piece_moves);
        });

        let match_result = Arbiter::arbitrate(&mut self.board, &move_list);
        match match_result {
            MatchResult::Draw => return 0,
            MatchResult::Loss => return -INFINITY + ply as i16,
            MatchResult::NoResult => {}
        }

        if depth == 0 {
            return self.quiescence_search(ply, 8, alpha, beta);
        }

        self.nodes += 1;

        let tt_entry = self.tt.probe(self.board.zobrist());
        if let Some(entry) = tt_entry
            && entry.depth >= depth
            && !ROOT
        {
            if self.board.is_repetition() {
                return 0;
            }

            match entry.tt_flag {
                TTFlag::Exact => {
                    if ply == 0 {
                        self.tt
                            .best_move
                            .store(entry.cur_move.packed_data(), Ordering::Relaxed);
                        self.tt.best_move_score.store(entry.eval, Ordering::Relaxed);
                    }

                    return Self::adjust_mate_distance(entry.eval, ply);
                }
                TTFlag::Upper => {
                    if entry.eval <= alpha {
                        return Self::adjust_mate_distance(alpha, ply);
                    }
                }
                TTFlag::Lower => {
                    if entry.eval >= beta {
                        return Self::adjust_mate_distance(beta, ply);
                    }
                }
            }
        }

        let pv_node = alpha != beta - 1;

        let last_move_was_null = {
            if let Some(last_move) = self.board.last_move() {
                last_move.is_default()
            } else {
                false
            }
        };

        if !self.board.in_check()
            && depth >= 3
            && !Self::in_zugzwang(&self.board)
            && !last_move_was_null
            && !pv_node
        {
            self.board.make_null_move();

            // let reduction = 3;

            // performs identically to the original reduction,
            // but may improve elo at higher depths in the future
            let reduction = 2 + depth / 5;
            let new_depth = depth.saturating_sub(reduction + 1);

            let score = -self.search::<NOT_ROOT>(ply + 1, new_depth, -beta, -beta + 1);

            if self.search_limits.is_hard_stop() {
                return 0;
            }

            self.board.undo_null_move();

            if score >= beta {
                return beta;
            }
        }

        // reverse futility pruning
        let static_eval = self.nnue.evaluate(&self.board);

        if !pv_node
            && !self.board.in_check()
            && static_eval >= (beta + 100 * depth as i16)
            && depth < 9
        {
            return beta;
        }

        // internal iterative reduction
        if depth > 3 && tt_entry.is_none() && !ROOT {
            depth -= 1;
        }

        self.order_moves(&mut move_list, &tt_entry, ply);

        let mut capture_moves = Vec::with_capacity(20);
        let mut quiet_moves = Vec::with_capacity(20);

        let mut node_type = TTFlag::Upper;
        let mut best_eval = -INFINITY;
        let mut best_move = move_list.move_at(0);

        for (move_count, curr_move) in move_list.iter().enumerate() {
            let is_capture = move_is_capture(&self.board, curr_move);
            // let reduction = LMR_REDUCTION.reduction(depth, move_count as u8);
            // let lmr_depth = (depth as i16 - reduction as i16 - 1).max(0) as u8;
            // let futility_margin = 120 + 100 * lmr_depth;

            // if !is_capture
            //     && !cur_move.flag().is_promotion()
            //     && !self.board.in_check()
            //     && !pv_node
            //     && !ROOT
            //     && depth <= 3
            //     && static_eval + futility_margin as i16 <= alpha
            // {
            //     continue;
            // }

            self.nnue.make_move(curr_move, &self.board);
            self.board.make_move::<true>(curr_move);

            self.search_stack[ply as usize] = *curr_move;

            let mut eval;

            let should_reduce =
                depth >= 3 && move_count >= 4 && !is_capture && !self.board.in_check();

            if should_reduce {
                let reduction = LMR_REDUCTION.reduction(depth, move_count as u8);
                let lmr_depth = (depth as i16 - reduction as i16 - 1).max(0) as u8;

                // let reduction = 1;
                eval = -self.search::<NOT_ROOT>(ply + 1, lmr_depth, -(alpha + 1), -alpha);

                if eval > alpha {
                    eval = -self.search::<NOT_ROOT>(ply + 1, depth - 1, -(alpha + 1), -alpha);

                    if eval > alpha && eval < beta {
                        eval = -self.search::<NOT_ROOT>(ply + 1, depth - 1, -beta, -alpha);
                    }
                }
            } else {
                eval = -self.search::<NOT_ROOT>(ply + 1, depth - 1, -(alpha + 1), -alpha);

                if eval > alpha && eval < beta {
                    eval = -self.search::<NOT_ROOT>(ply + 1, depth - 1, -beta, -alpha);
                }
            }

            if self.search_limits.is_hard_stop() {
                return 0;
            }

            self.board.undo_move();
            self.nnue.undo_move();

            if eval >= beta {
                self.tt.update(
                    self.board.zobrist(),
                    *curr_move,
                    beta,
                    depth,
                    TTFlag::Lower,
                    false,
                    ply,
                );

                // self.capture_heuristics
                //     .update(&self.board, &cur_move, &capture_moves, depth);

                if !is_capture {
                    self.killers.update(*curr_move, ply);
                    self.history
                        .update(*curr_move, &quiet_moves, depth, &self.board);

                    self.counter_move_heuristics.update(
                        *curr_move,
                        &quiet_moves,
                        depth,
                        self.board.side_to_move(),
                    );
                }

                if ROOT {
                    self.tt
                        .best_move
                        .store(curr_move.packed_data(), Ordering::Relaxed);
                    self.tt.best_move_score.store(beta, Ordering::Relaxed);
                }

                return beta;
            }
            if eval > alpha {
                alpha = eval;
                node_type = TTFlag::Exact;
                best_move = *curr_move;
            }

            if eval > best_eval {
                best_eval = eval;
                best_move = *curr_move;
            }

            if move_is_capture(&self.board, curr_move) {
                capture_moves.push(*curr_move);
            } else if !curr_move.flag().is_promotion() {
                quiet_moves.push(*curr_move);
            }
        }

        self.tt.update(
            self.board.zobrist(),
            best_move,
            alpha,
            depth,
            node_type,
            pv_node,
            ply,
        );

        if ROOT && matches!(node_type, TTFlag::Exact) {
            self.tt
                .best_move
                .store(best_move.packed_data(), Ordering::Relaxed);
            self.tt.best_move_score.store(alpha, Ordering::Relaxed);
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

    fn quiescence_search(&mut self, ply: u8, depth: u8, mut alpha: i16, beta: i16) -> i16 {
        if self.search_limits.is_hard_stop() {
            return 0;
        }

        self.nodes += 1;
        let pv_node = alpha != beta - 1;

        let tt_entry = self.tt.probe(self.board.zobrist());
        if let Some(entry) = tt_entry {
            if !pv_node && entry.depth >= depth {
                match entry.tt_flag {
                    TTFlag::Exact => return Self::adjust_mate_distance(entry.eval, ply),
                    TTFlag::Upper => {
                        if entry.eval <= alpha {
                            return Self::adjust_mate_distance(alpha, ply);
                        }
                    }
                    TTFlag::Lower => {
                        if entry.eval >= beta {
                            return Self::adjust_mate_distance(beta, ply);
                        }
                    }
                }
            }
        }

        let eval = self.nnue.evaluate(&self.board);

        if eval >= beta {
            return beta;
        }
        if alpha < eval {
            alpha = eval
        }

        if depth == 0 {
            return alpha;
        }

        let mut all_move_list = MoveList::default();
        MoveGenerator::<GEN_ALL>::generate::<NOT_LEAF>(&mut self.board, &mut |mut piece_moves| {
            all_move_list.add_piece_moves(&mut piece_moves);
        });

        let match_result = Arbiter::arbitrate(&self.board, &all_move_list);
        match match_result {
            MatchResult::Draw => return 0,
            MatchResult::Loss => return -INFINITY + ply as i16,
            MatchResult::NoResult => {}
        }

        let mut tactial_move_list = MoveList::default();
        MoveGenerator::<GEN_TACTICS>::generate::<NOT_LEAF>(
            &mut self.board,
            &mut |mut piece_moves| {
                tactial_move_list.add_piece_moves(&mut piece_moves);
            },
        );

        let mut node_type = TTFlag::Upper;
        let mut best_move = tactial_move_list.move_at(0);

        for cur_move in tactial_move_list.iter() {
            self.nnue.make_move(cur_move, &mut self.board);
            self.board.make_move::<true>(cur_move);

            let eval = -self.quiescence_search(ply + 1, depth - 1, -beta, -alpha);
            if self.search_limits.is_hard_stop() {
                return 0;
            }

            self.board.undo_move();
            self.nnue.undo_move();

            if eval > alpha {
                best_move = *cur_move;
                node_type = TTFlag::Exact;
                alpha = eval;
            }

            if eval >= beta {
                /* self.tt.update(
                    self.board.zobrist(),
                    *cur_move,
                    alpha,
                    0,
                    TTFlag::Lower,
                    false,
                    ply,
                ); */
                node_type = TTFlag::Lower;
                alpha = beta;
                break;

                // return beta;
            }
        }

        self.tt.update(
            self.board.zobrist(),
            best_move,
            alpha,
            0,
            node_type,
            false,
            ply,
        );

        alpha
    }

    fn pv_from_transposition(&self) -> String {
        let mut board_clone = self.board.clone();
        let mut pv_line = String::new();

        loop {
            if pv_line.chars().filter(|&cur_char| ' ' == cur_char).count() == 20 {
                break;
            }

            if let Some(entry) = self.tt.probe(board_clone.zobrist()) {
                let best_move = entry.cur_move;

                let mut valid_moves = MoveList::default();
                MoveGenerator::<GEN_ALL>::generate::<NOT_LEAF>(
                    &mut board_clone,
                    &mut |mut piece_moves| {
                        valid_moves.add_piece_moves(&mut piece_moves);
                    },
                );

                if !valid_moves.contains_move(best_move) {
                    break;
                }

                let match_result = Arbiter::arbitrate(&board_clone, &valid_moves);

                if !matches!(match_result, MatchResult::NoResult) || board_clone.is_repetition() {
                    break;
                }

                pv_line += &format!("{best_move} ");
                board_clone.make_move::<true>(&best_move);
            } else {
                break;
            }
        }

        pv_line
    }

    fn aspiration_windows(&mut self, depth: u8) {
        let mut alpha = -INFINITY;
        let mut beta = INFINITY;
        let mut delta = 35;

        if depth >= 5 {
            let current_eval = self.tt.best_move_score.load(Ordering::Relaxed);
            alpha = current_eval - delta;
            beta = current_eval + delta;
        }

        loop {
            let eval = self.search::<IS_ROOT>(0, depth, alpha, beta);

            if self.search_limits.is_hard_stop() {
                return;
            }

            if eval <= alpha {
                beta = (alpha + beta) / 2;
                alpha = (-INFINITY).max(alpha - delta)
            } else if eval >= beta {
                beta += delta;
            } else {
                break;
            }

            delta += delta / 2;
            if delta >= 1000 {
                alpha = -INFINITY;
                beta = INFINITY;
            }
        }
    }
    pub fn iterative_deepening(&mut self) -> MovePly {
        for cur_depth in (1..64).step_by(1) {
            self.aspiration_windows(cur_depth);

            if self.tt.curr_depth.load(Ordering::SeqCst) >= cur_depth {
                continue;
            }

            self.tt.curr_depth.add(1, Ordering::SeqCst);

            // self.search::<IS_ROOT>(0, cur_depth, -INFINITY, INFINITY);
            if self.search_limits.is_hard_stop() {
                break;
            }

            let pv_line = self.pv_from_transposition();
            // let sel_depth = pv_line.split(" ").collect::<Vec<&str>>().len() - 1;
            let time = self.search_limits.ms_elapsed();
            let eval = self.tt.best_move_score.load(Ordering::Relaxed);
            let nodes = self.nodes;
            let nps = (nodes as f64 / (time as f64 / 1000f64).max(0.0001f64)) as u128;
            let hash_full = self.tt.hash_full();

            println!(
                "info depth {cur_depth} score cp {eval} nodes {nodes} nps {nps} time {time} hashfull {hash_full} pv {pv_line}"
            );

            if self.search_limits.is_soft_stop() {
                break;
            }
        }

        self.tt.best_move.load(Ordering::Relaxed).into()
    }

    fn mvv_lva(&self, curr_move: &MovePly) -> i16 {
        if !self.board.piece_at(curr_move.to()).is_piece() {
            return 0;
        }

        let us_value = PIECE_VALUES[self.board.piece_at(curr_move.from()) as usize];
        let them_value = PIECE_VALUES[self.board.piece_at(curr_move.to()) as usize];

        them_value - us_value / 16
    }

    fn order_moves(
        &mut self,
        move_list: &mut MoveList,
        prev_best_move: &Option<TTEntry>,
        ply_searched: u8,
    ) {
        let mut move_values: [i16; 256] = [0; 256];

        let potential_entry = self.tt.probe(self.board.zobrist());

        for (i, curr_move) in move_list.iter().enumerate() {
            if let Some(entry) = potential_entry {
                if entry.cur_move == *curr_move {
                    move_values[i] = INFINITY;
                    continue;
                }
            }

            let flag = curr_move.flag();

            if let Some(entry) = prev_best_move {
                if entry.cur_move == *curr_move {
                    move_values[i] = INFINITY;
                    continue;
                }
            }

            if self.board.piece_at(curr_move.to()).is_piece() {
                move_values[i] += self.mvv_lva(curr_move) + 16000;
            } else {
                move_values[i] += self
                    .history
                    .get(self.board.piece_at(curr_move.from()), curr_move.to())
                    / 2;

                // move_values[i] += self
                //     .counter_move_heuristics
                //     .get(*curr_move, self.board.side_to_move())
                //     / 2;
            }

            if self.killers.is_first_slot(*curr_move, ply_searched) {
                move_values[i] += 10000;
            }

            if self.killers.is_second_slot(*curr_move, ply_searched) {
                move_values[i] += 8000;
            }

            if flag.is_promotion() {
                move_values[i] += PIECE_VALUES[flag.promotion_piece(White) as usize] * 10;
            }

            if flag.is_castles() {
                move_values[i] += 5000;
            }
        }

        move_list.order_moves(&move_values);
    }

    pub fn search_start(
        tt: &Arc<Transposition>,
        board: &Board,
        search_limits: &SearchLimits,
        num_threads: usize,
    ) {
        let limits_copy = search_limits.clone();
        let board_copy = board.clone();

        let mut handles = Vec::new();
        for i in 0..num_threads {
            let tt_copy = Arc::clone(tt);

            let handle = thread::Builder::new()
                .stack_size(32 * 1024 * 1024)
                .name(format!("Engine Thread {i}"))
                .spawn(move || {
                    let thread_board = board_copy.clone();
                    let thread_limits = limits_copy.clone();
                    let mut searcher = Searcher::new(&tt_copy, &thread_board, &thread_limits);

                    searcher.iterative_deepening();
                });

            handles.push(handle.unwrap());
        }

        for handle in handles {
            handle.join().unwrap();
        }

        let best_move: MovePly = tt.best_move.load(Ordering::Relaxed).into();

        println!("bestmove {}\n", best_move);

        tt.curr_depth.store(0, Ordering::Relaxed);
        tt.age();
    }
}
