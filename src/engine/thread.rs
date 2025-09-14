use crate::engine::history_heuristics::HistoryHeuristics;
use crate::engine::killers::Killers;


pub struct ThreadData{
    pub killers: Killers,
    pub history_heuristics: HistoryHeuristics,
    pub nodes: u128,
}

impl Default for ThreadData {
    fn default() -> Self {
        Self{
            killers: Killers::default(),
            history_heuristics: HistoryHeuristics::default(),
            nodes: 0,
        }
    }
}
