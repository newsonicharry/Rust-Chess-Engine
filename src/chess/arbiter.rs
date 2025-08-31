use crate::chess::board::Board;
use crate::chess::types::match_result::MatchResult;

pub struct Arbiter{}

impl Arbiter{
    pub fn arbitrate(board: &Board) -> MatchResult {
        MatchResult::NoResult
    }
}