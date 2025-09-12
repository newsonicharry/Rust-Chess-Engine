use crate::chess::move_ply::MovePly;

pub enum Commands {
    Uci,
    IsReady,
    UciNewGame,
    Position { fen: String, moves: Option<Vec<String>> },
    Go { move_time: Option<u32>, wtime: Option<u32>, btime: Option<u32>, winc: Option<u32>, binc: Option<u32>, moves_to_go: Option<u32>},
    SetOption { options_type: OptionsType },
    Stop,
    Quit,
    Help,
    Perft {depth: u32},
    Unknown(String),
    IncorrectFormat,
}

pub enum OptionsType {
    Spin { name: String, value: u16 },
    Button { name: String },
}