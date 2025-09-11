use crate::chess::move_ply::MovePly;
use crate::uci::option_table::OptionType;

pub enum Commands {
    Uci,
    IsReady,
    UciNewGame,
    Position { fen: String, moves: Option<Vec<String>> },
    Go { move_time: Option<u32>, wtime: Option<u32>, btime: Option<u32>, winc: Option<u32>, binc: Option<u32>, moves_to_go: Option<u32>},
    SetOption { name: String, option_type: OptionType, value: Option<String> },
    Stop,
    Quit,
    Help,
    Perft {depth: u32},
    Unknown(String),
    IncorrectFormat,
}