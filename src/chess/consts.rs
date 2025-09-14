pub const NUM_SQUARES: usize = 64;
pub const NUM_PIECES: usize = 12;
pub const NUM_FILES: usize = 8;
pub const NUM_RANKS: usize = 8;

pub const MAX_MOVES: usize = 1024;
pub const MAX_DEPTH: usize = 256;

pub const PIECE_VALUES: [i16; 12] = [100, 320, 320, 500, 1000, 10000, 100, 320, 320, 500, 1000, 10000];

pub const KNIGHT_DIRECTIONS: [(i8,i8); 8] = [(-2, 1), (-1, 2), (1, 2), (2, 1), (2, -1), (1, -2), (-1, -2), (-2, -1)];
pub const BISHOP_DIRECTIONS: [(i8,i8); 4] = [(1, 1), (-1, 1), (-1, -1), (1, -1)];
pub const ROOK_DIRECTIONS: [(i8,i8); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];
pub const QUEEN_DIRECTIONS: [(i8,i8); 8] = [(1, 1), (-1, 1), (-1, -1), (1, -1), (1, 0), (-1, 0), (0, 1), (0, -1)];
pub const KING_DIRECTIONS: [(i8,i8); 8] = [(-1, 1), (0, 1), (1, 1), (1, 0), (1, -1), (0, -1), (-1, -1), (-1, 0)];

pub const WHITE_PAWN_ATTACKS_DIRECTIONS: [(i8,i8); 2] = [(1, 1), (-1, 1)];
pub const BLACK_PAWN_ATTACKS_DIRECTIONS: [(i8,i8); 2] = [(1, -1), (-1, -1)];

pub const NUM_ORTHOGONAL_ENTRIES: usize = 102400;
pub const NUM_DIAGONAL_ENTRIES: usize = 5248;

pub const USE_BMI2: bool = true;
