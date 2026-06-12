use crate::chess::board::Board;
use crate::chess::consts;
use crate::chess::consts::PIECE_VALUES;
use crate::chess::move_generator::MoveGenerator;
use crate::chess::move_generator::{GEN_ALL, GEN_TACTICS};
use crate::chess::move_list::MoveList;
use crate::chess::move_ply::MovePly;
use crate::chess::types::color::Color::White;
use crate::chess::types::piece::BasePiece::{King, Pawn};
use crate::engine::arbiter::Arbiter;
use crate::engine::capture_history::CaptureHeuristics;
use crate::engine::counter_move_heuristics::CounterMoveHeuristics;
use crate::engine::eval::nnue::NNUE;
use crate::engine::history_heuristics::HistoryHeuristics;
use crate::engine::killers::Killers;
use crate::engine::search_funcs::{move_is_capture, move_is_quiet, see};
use crate::engine::search_limits::SearchLimits;
use crate::engine::thread::ThreadData;
use crate::engine::transposition::{TTEntry, Transposition};
use crate::engine::types::match_result::MatchResult;
use crate::engine::types::tt_flag::TTFlag;
use crate::precomputed::accessor::LMR_REDUCTION;
use std::sync::Arc;
use std::sync::atomic::Ordering;

const INFINITY: i16 = 30000;

pub struct Searcher {
    board: Board,
    search_stack: [MovePly; consts::MAX_DEPTH],
    killers: Killers,
    capture_heuristics: CaptureHeuristics,
    history_heuristics: HistoryHeuristics,
    counter_move_heuristics: CounterMoveHeuristics,
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
            search_stack: [MovePly::default(); consts::MAX_DEPTH],
            killers: Killers::default(),
            capture_heuristics: CaptureHeuristics::default(),
            history_heuristics: HistoryHeuristics::default(),
            counter_move_heuristics: CounterMoveHeuristics::default(),
            nodes: 0,
            tt: Arc::clone(transposition),
            nnue: NNUE::new(board.clone()),
            search_limits: search_limits.clone(),
        }
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

        self.nodes += 1;

        let tt_entry = self.tt.probe(self.board.zobrist());
        if let Some(entry) = tt_entry
            && entry.depth >= depth
        {
            match entry.tt_flag {
                TTFlag::Exact => {
                    if ply == 0 {
                        self.tt
                            .best_move
                            .store(entry.cur_move.packed_data(), Ordering::Relaxed);
                        self.tt.best_move_score.store(entry.eval, Ordering::Relaxed);
                    }

                    return entry.eval;
                }
                TTFlag::Upper => {
                    if entry.eval <= alpha {
                        return entry.eval;
                    }
                }
                TTFlag::Lower => {
                    if entry.eval >= beta {
                        return entry.eval;
                    }
                }
            }
        }

        let mut move_list = MoveList::default();
        MoveGenerator::<GEN_ALL>::generate(&mut self.board, &mut move_list);

        let match_result = Arbiter::arbitrate(&mut self.board, &move_list);
        match match_result {
            MatchResult::Draw => return -50,
            MatchResult::Loss => return -INFINITY + ply as i16,
            MatchResult::NoResult => {}
        }

        if self.board.in_check() {
            depth += 1;
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

            let reduction = 3;

            let new_depth = (depth as i16 - 1 - reduction).max(0) as u8;
            let score = -self.search::<NOT_ROOT>(ply + 1, new_depth, -beta, -beta + 1);

            if self.search_limits.is_hard_stop() {
                return 0;
            }

            self.board.undo_null_move();

            if score >= beta {
                return beta;
            }
        }

        if depth == 0 {
            return self.quiescence_search(ply, 8, alpha, beta);
            // return self.nnue.evaluate(self.board.side_to_move());
        }

        // reverse futility pruning
        let static_eval = self.nnue.evaluate(self.board.side_to_move());

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

        for (move_count, cur_move) in move_list.iter().enumerate() {
            let is_capture = move_is_capture(&self.board, cur_move);
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

            self.nnue.make_move(cur_move, &self.board);
            self.board.make_move(cur_move);

            self.search_stack[ply as usize] = *cur_move;

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
                    *cur_move,
                    beta,
                    depth,
                    TTFlag::Lower,
                    false,
                );

                self.capture_heuristics
                    .update(&self.board, &cur_move, &capture_moves, depth);

                self.killers.update(*cur_move, depth);

                self.history_heuristics.update(
                    cur_move,
                    &quiet_moves,
                    self.board.side_to_move(),
                    depth,
                );

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

            if move_is_capture(&self.board, cur_move) {
                capture_moves.push(*cur_move);
            } else if !cur_move.flag().is_promotion() {
                quiet_moves.push(*cur_move);
            }
        }

        self.tt.update(
            self.board.zobrist(),
            best_move,
            alpha,
            depth,
            node_type,
            pv_node,
        );

        if ply == 0 {
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

    fn quiescence_search(&mut self, ply_searched: u8, depth: u8, mut alpha: i16, beta: i16) -> i16 {
        if self.search_limits.is_hard_stop() {
            return 0;
        }

        self.nodes += 1;
        let pv_node = alpha != beta - 1;

        let tt_entry = self.tt.probe(self.board.zobrist());
        if let Some(entry) = tt_entry {
            if !pv_node && entry.depth >= depth {
                match entry.tt_flag {
                    TTFlag::Exact => return entry.eval,
                    TTFlag::Upper => {
                        if entry.eval <= alpha {
                            return entry.eval;
                        }
                    }
                    TTFlag::Lower => {
                        if entry.eval >= beta {
                            return entry.eval;
                        }
                    }
                }
            }
        }

        let eval = self.nnue.evaluate(self.board.side_to_move());

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
        MoveGenerator::<GEN_ALL>::generate(&mut self.board, &mut all_move_list);

        let match_result = Arbiter::arbitrate(&self.board, &all_move_list);
        match match_result {
            MatchResult::Draw => return 0,
            MatchResult::Loss => return -INFINITY + ply_searched as i16,
            MatchResult::NoResult => {}
        }

        let mut tactial_move_list = MoveList::default();
        MoveGenerator::<GEN_TACTICS>::generate(&mut self.board, &mut tactial_move_list);

        let mut node_type = TTFlag::Upper;
        let mut best_move = tactial_move_list.move_at(0);

        for cur_move in tactial_move_list.iter() {
            self.nnue.make_move(cur_move, &mut self.board);
            self.board.make_move(cur_move);

            let eval = -self.quiescence_search(ply_searched + 1, depth - 1, -beta, -alpha);
            if self.search_limits.is_hard_stop() {
                return 0;
            }

            self.board.undo_move();
            self.nnue.undo_move();

            if eval >= beta {
                self.tt.update(
                    self.board.zobrist(),
                    *cur_move,
                    alpha,
                    0,
                    TTFlag::Lower,
                    false,
                );
                return beta;
            }

            if eval > alpha {
                best_move = *cur_move;
                node_type = TTFlag::Exact;
                alpha = eval;
            }
        }

        self.tt
            .update(self.board.zobrist(), best_move, alpha, 0, node_type, false);

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
                MoveGenerator::<GEN_ALL>::generate(&mut board_clone, &mut valid_moves);

                if !valid_moves.contains_move(best_move) {
                    break;
                }

                pv_line += &format!("{best_move} ");
                board_clone.make_move(&best_move);
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
        for cur_depth in (1..32).step_by(1) {
            self.aspiration_windows(cur_depth);

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

    fn order_moves(
        &mut self,
        move_list: &mut MoveList,
        prev_best_move: &Option<TTEntry>,
        ply_searched: u8,
    ) {
        let mut move_values: [i16; 256] = [0; 256];

        // let mut counter_move = MovePly::default();
        // let last_played = self.board.last_move();

        // if let Some(last_move) = last_played {
        //     counter_move = self
        //         .counter_move_heuristics
        //         .get_counter_move(last_move, self.board.side_to_move());
        // }

        for (i, cur_move) in move_list.iter().enumerate() {
            // self.board.make_move(cur_move);
            // if self.tt.probe(self.board.zobrist()).is_some() {
            //     move_values[i] = 30000;
            //     self.board.undo_move();
            //     continue;
            // }
            // self.board.undo_move();

            let flag = cur_move.flag();

            if let Some(entry) = prev_best_move {
                if entry.cur_move == *cur_move {
                    move_values[i] = INFINITY;
                    continue;
                }
            }

            move_values[i] += self.capture_heuristics.get(
                cur_move.to(),
                self.board.piece_at(cur_move.from()),
                self.board.piece_at(cur_move.to()),
            );

            // if *cur_move == counter_move {
            //     move_values[i] += 300;
            // }

            // 8 1386
            // 1396

            // if self.killers.contains(ply_searched, *cur_move) {
            //     move_values[i] += 5000;
            // }

            move_values[i] += 2 * see(cur_move.from(), cur_move.to(), &self.board);

            // let attacker_value = PIECE_VALUES[self.board.piece_at(cur_move.from()) as usize];
            // let victim_value = PIECE_VALUES[self.board.piece_at(cur_move.to()) as usize];
            // if self.board.piece_at(cur_move.to()).is_piece() {
            //     move_values[i] += victim_value - attacker_value;
            // }

            // 1625, 7701 depth 14
            //

            if flag.is_promotion() {
                move_values[i] += PIECE_VALUES[flag.promotion_piece(White) as usize] * 10;
            }

            if flag.is_castles() {
                move_values[i] += 1000;
            }
            // if move_is_quiet(&self.board, cur_move) {
            //     move_values[i] += self
            //         .history_heuristics
            //         .get(*cur_move, self.board.side_to_move());
            // }
        }

        move_list.order_moves(&move_values);
    }

    pub fn search_start(&mut self, _num_threads: usize) {
        // let mut nnue = NNUE::default();
        // nnue.new(&mut board);

        // let mut thread_board = board.clone();

        // let tt_new? = Arc::clone(&tt);

        // let best_move = self.iterative_deepening(&mut thread_board, &tt, &mut nnue, &search_limits);
        let best_move = self.iterative_deepening();
        self.tt.age();
        println!("bestmove {}\n", best_move);

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
}
