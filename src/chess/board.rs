use crate::chess::bitboard::Bitboard;
use crate::chess::consts::{MAX_MOVES, NUM_BASE_PIECES, NUM_PIECES, NUM_SQUARES};
use crate::chess::move_generator::{BLACK, WHITE};
use crate::chess::move_ply::MovePly;
use crate::chess::types::color::Color;
use crate::chess::types::file::File;
use crate::chess::types::move_flag::MoveFlag;
use crate::chess::types::piece::Piece::{
    BlackBishop, BlackKing, BlackPawn, BlackQueen, BlackRook, NoPiece, WhiteBishop, WhiteKing,
    WhitePawn, WhiteQueen, WhiteRook,
};
use crate::chess::types::piece::{
    BISHOP, BasePiece, ITER_BLACK, ITER_WHITE, KING, KNIGHT, PAWN, Piece, QUEEN, ROOK,
    basepiece_to_piece, char_to_piece,
};
use crate::chess::types::rank::Rank;
use crate::chess::types::square::Square;
use crate::precomputed::accessor::ZOBRIST;
use std::fmt::Display;

// if a piece on a certain square moves then the castling rights must change as well
const SQUARE_MOVED_CASTLING: [u8; NUM_SQUARES] = [
    13, 15, 15, 15, 12, 15, 15, 14, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15,
    15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15,
    15, 15, 15, 15, 15, 15, 15, 15, 7, 15, 15, 15, 3, 15, 15, 11,
];
pub const INCREMENT_ZOBRIST: bool = true;
pub const NO_ZOBRIST: bool = false;

#[derive(Copy, Clone)]
pub struct Board {
    bitboards: [Bitboard; NUM_PIECES],
    side_to_move: Color,

    white_occupancy: u64,
    black_occupancy: u64,
    occupancy: u64,

    en_passant_file: File,
    can_en_passant: bool,
    castling_rights: u8,
    half_move_clock: u8,
    zobrist: u64,
    in_check: bool,
}

impl Default for Board {
    fn default() -> Board {
        let board = Board {
            bitboards: [Bitboard::default(); NUM_PIECES],
            side_to_move: Color::White,

            white_occupancy: 0,
            black_occupancy: 0,
            occupancy: 0,

            en_passant_file: File::default(),
            can_en_passant: false,
            castling_rights: 0,
            half_move_clock: 0,
            zobrist: 0,
            in_check: false,
        };

        board
    }
}

impl Board {
    // fen string is considered accurate at this point
    // determining the fen strings accuracy is the job of the uci
    pub fn new(&mut self, fen: &str) {
        let split_fen: Vec<&str> = fen.split_whitespace().collect();

        let fen_sequence = split_fen.get(0).unwrap();
        let ranks = fen_sequence.split('/').collect::<Vec<&str>>();

        for (i, section) in ranks.iter().rev().enumerate() {
            let rank = Rank::from(i as u8);

            let mut cur_file = File::A;
            for char in section.chars() {
                if char.is_numeric() {
                    let num_skipped_files = char.to_digit(10).unwrap() as u8;
                    cur_file.add(num_skipped_files);
                    continue;
                }

                let cur_square = Square::from((cur_file, rank));
                cur_file.plus();
                let piece = char_to_piece(char).unwrap();

                match piece {
                    Piece::WhitePawn => self.add_piece::<PAWN, WHITE, NO_ZOBRIST>(cur_square),
                    Piece::WhiteKnight => self.add_piece::<KNIGHT, WHITE, NO_ZOBRIST>(cur_square),
                    Piece::WhiteBishop => self.add_piece::<BISHOP, WHITE, NO_ZOBRIST>(cur_square),
                    Piece::WhiteRook => self.add_piece::<ROOK, WHITE, NO_ZOBRIST>(cur_square),
                    Piece::WhiteQueen => self.add_piece::<QUEEN, WHITE, NO_ZOBRIST>(cur_square),
                    Piece::WhiteKing => self.add_piece::<KING, WHITE, NO_ZOBRIST>(cur_square),

                    Piece::BlackPawn => self.add_piece::<PAWN, BLACK, NO_ZOBRIST>(cur_square),
                    Piece::BlackKnight => self.add_piece::<KNIGHT, BLACK, NO_ZOBRIST>(cur_square),
                    Piece::BlackBishop => self.add_piece::<BISHOP, BLACK, NO_ZOBRIST>(cur_square),
                    Piece::BlackRook => self.add_piece::<ROOK, BLACK, NO_ZOBRIST>(cur_square),
                    Piece::BlackQueen => self.add_piece::<QUEEN, BLACK, NO_ZOBRIST>(cur_square),
                    Piece::BlackKing => self.add_piece::<KING, BLACK, NO_ZOBRIST>(cur_square),

                    Piece::NoPiece => {}
                }
            }
        }
        let side_to_move_str = split_fen.get(1).unwrap();
        if side_to_move_str.contains("b") {
            self.side_to_move = Color::Black;
        }

        let castling_rights_str = split_fen.get(2).unwrap();
        if castling_rights_str.contains("K") {
            self.castling_rights |= 0b0001
        }
        if castling_rights_str.contains("Q") {
            self.castling_rights |= 0b0010
        }
        if castling_rights_str.contains("k") {
            self.castling_rights |= 0b0100
        }
        if castling_rights_str.contains("q") {
            self.castling_rights |= 0b1000
        }

        self.update_occupancy();
        self.zobrist = ZOBRIST.hash_from_board(&self);
    }

    /* pub fn is_repetition(&self) -> bool {
        let Some(board_states) = self.past_board_states() else {
            return false;
        };

        for board_state in board_states.iter().rev().skip(0).step_by(1) {
            if board_state.zobrist == self.zobrist {
                return true;
            }

            if self.half_move_clock == 0 {
                return false;
            }
        }
        return false;
    } */

    pub fn set_in_check(&mut self, in_check: bool) {
        self.in_check = in_check;
    }

    pub fn in_check(&self) -> bool {
        self.in_check
    }

    /* pub fn last_move(&self) -> Option<MovePly> {
        if self.cur_board_state == 0 {
            return None;
        }

        let last_move = self.board_states[self.cur_board_state - 1].played;
        Some(last_move)
    } */

    pub fn orthogonal_bitboard_them<const COLOR: bool>(&self) -> u64 {
        match COLOR {
            WHITE => self.bitboards[BlackRook as usize].0 | self.bitboards[BlackQueen as usize].0,
            BLACK => self.bitboards[WhiteRook as usize].0 | self.bitboards[WhiteQueen as usize].0,
        }
    }

    pub fn diagonal_bitboard_them<const COLOR: bool>(&self) -> u64 {
        match COLOR {
            WHITE => self.bitboards[BlackBishop as usize].0 | self.bitboards[BlackQueen as usize].0,
            BLACK => self.bitboards[WhiteBishop as usize].0 | self.bitboards[WhiteQueen as usize].0,
        }
    }

    pub fn bitboard_const<const COLOR: bool>(&self, base_piece: BasePiece) -> u64 {
        let base_piece = basepiece_to_piece::<COLOR>(base_piece);
        self.bitboards[base_piece as usize].0
    }

    pub fn bitboard(&self, base_piece: BasePiece, color: Color) -> u64 {
        match color {
            Color::White => self.bitboards[base_piece as usize].0,
            Color::Black => self.bitboards[base_piece as usize + 6].0,
        }
    }

    pub fn bitboard_them(&self, base_piece: BasePiece) -> u64 {
        let piece = Piece::from((base_piece, !self.side_to_move));
        self.bitboards[piece as usize].0
    }

    pub fn bitboard_combined(&self, piece: BasePiece) -> u64 {
        self.bitboards[piece as usize].0 | self.bitboards[piece as usize + 6].0
    }

    pub fn piece_at(&self, square: Square) -> Piece {
        let mask = square.mask();
        for index in (0..NUM_PIECES) {
            if self.bitboards[index] & mask != 0 {
                return unsafe { std::mem::transmute(index as u8) };
            }
        }

        return Piece::NoPiece;
        // self.piece_squares[square as usize]
    }

    pub fn king_square_const<const COLOR: bool>(&self) -> Square {
        match COLOR {
            WHITE => self.king_square(Color::White),
            BLACK => self.king_square(Color::Black),
        }
    }

    pub fn king_square(&self, color: Color) -> Square {
        match color {
            Color::White => self.bitboards[WhiteKing as usize].0.trailing_zeros().into(),
            Color::Black => self.bitboards[BlackKing as usize].0.trailing_zeros().into(),
        }
    }

    pub fn occupancy_us(&self) -> u64 {
        match self.side_to_move {
            Color::White => self.white_occupancy,
            Color::Black => self.black_occupancy,
        }
    }

    pub fn occupancy_them(&self) -> u64 {
        match self.side_to_move {
            Color::White => self.black_occupancy,
            Color::Black => self.white_occupancy,
        }
    }

    pub fn occupancy_them_const<const COLOR: bool>(&self) -> u64 {
        match COLOR {
            WHITE => self.black_occupancy,
            BLACK => self.white_occupancy,
        }
    }

    pub fn occupancy_us_const<const COLOR: bool>(&self) -> u64 {
        match COLOR {
            WHITE => self.white_occupancy,
            BLACK => self.black_occupancy,
        }
    }

    pub fn occupancy(&self) -> u64 {
        self.occupancy
    }

    pub fn side_to_move(&self) -> Color {
        self.side_to_move
    }

    pub fn en_passant_file(&self) -> Option<File> {
        match self.can_en_passant {
            true => Some(self.en_passant_file),
            false => None,
        }
    }

    pub fn has_short_castle_rights_const<const COLOR: bool>(&self) -> bool {
        match COLOR {
            WHITE => self.castling_rights & 0b0001 != 0,
            BLACK => self.castling_rights & 0b0100 != 0,
        }
    }

    pub fn has_long_castle_rights_const<const COLOR: bool>(&self) -> bool {
        match COLOR {
            WHITE => self.castling_rights & 0b0010 != 0,
            BLACK => self.castling_rights & 0b1000 != 0,
        }
    }

    pub fn has_short_castle_rights(&self, color: Color) -> bool {
        match color {
            Color::White => self.castling_rights & 0b0001 != 0,
            Color::Black => self.castling_rights & 0b0100 != 0,
        }
    }

    pub fn has_long_castle_rights(&self, color: Color) -> bool {
        match color {
            Color::White => self.castling_rights & 0b0010 != 0,
            Color::Black => self.castling_rights & 0b1000 != 0,
        }
    }

    pub fn zobrist(&self) -> u64 {
        self.zobrist
    }

    /* pub fn past_board_states(&self) -> Option<&[BoardState]> {
        if self.cur_board_state > 0 {
            return Some(&self.board_states[..(self.cur_board_state)]);
        }

        None
    } */

    pub fn half_move_clock(&self) -> u8 {
        self.half_move_clock
    }

    pub fn update_occupancy(&mut self) {
        self.white_occupancy = 0;
        self.black_occupancy = 0;
        self.occupancy = 0;

        for piece in Piece::iterator::<ITER_WHITE>() {
            let bitboard = self.bitboards[piece as usize].0;
            self.white_occupancy |= bitboard;
            self.occupancy |= bitboard;
        }

        for piece in Piece::iterator::<ITER_BLACK>() {
            let bitboard = self.bitboards[piece as usize].0;
            self.black_occupancy |= bitboard;
            self.occupancy |= bitboard;
        }
    }

    #[inline(always)]
    fn add_piece<const PIECE: u8, const COLOR: bool, const ZOBRIST_OPTION: bool>(
        &mut self,
        square: Square,
    ) {
        let piece = Piece::from((PIECE.into(), COLOR.into()));

        self.bitboards[piece as usize].add_piece(square);

        match COLOR {
            WHITE => self.white_occupancy ^= square.mask(),
            BLACK => self.black_occupancy ^= square.mask(),
        }

        if ZOBRIST_OPTION == INCREMENT_ZOBRIST {
            self.zobrist ^= ZOBRIST.square_zobrist(piece, square);
        }
    }
    #[inline(always)]
    fn remove_piece<const PIECE: u8, const COLOR: bool, const ZOBRIST_OPTION: bool>(
        &mut self,
        square: Square,
    ) {
        let piece = Piece::from((PIECE.into(), COLOR.into()));

        self.bitboards[piece as usize].remove_piece(square);

        match COLOR {
            WHITE => self.white_occupancy ^= square.mask(),
            BLACK => self.black_occupancy ^= square.mask(),
        }

        if ZOBRIST_OPTION == INCREMENT_ZOBRIST {
            self.zobrist ^= ZOBRIST.square_zobrist(piece, square);
        }
    }
    #[inline(always)]
    fn move_piece<const COLOR: bool, const ZOBRIST_OPTION: bool>(
        &mut self,
        from: Square,
        to: Square,
        moving_piece: BasePiece,
    ) {
        let piece = basepiece_to_piece::<COLOR>(moving_piece);
        self.bitboards[piece as usize].move_piece(from, to);

        match COLOR {
            WHITE => {
                self.white_occupancy ^= from.mask();
                self.white_occupancy ^= to.mask();
            }
            BLACK => {
                self.black_occupancy ^= from.mask();
                self.black_occupancy ^= to.mask();
            }
        }

        if ZOBRIST_OPTION == INCREMENT_ZOBRIST {
            self.zobrist ^= ZOBRIST.square_zobrist(piece, from);
            self.zobrist ^= ZOBRIST.square_zobrist(piece, to);
        }
    }

    #[inline(always)]
    fn apply_quiet<const COLOR: bool, const ZOBRIST_OPTION: bool>(
        &mut self,
        played: &MovePly,
        piece: BasePiece,
    ) {
        self.move_piece::<COLOR, ZOBRIST_OPTION>(played.from(), played.to(), piece)
    }

    #[inline(always)]
    fn apply_double_jump<const COLOR: bool, const ZOBRIST_OPTION: bool>(
        &mut self,
        played: &MovePly,
        piece: BasePiece,
    ) {
        self.en_passant_file = played.from().file();
        self.can_en_passant = true;

        if ZOBRIST_OPTION == INCREMENT_ZOBRIST {
            self.zobrist ^= ZOBRIST.pawn_jump(self.en_passant_file);
        }
        self.apply_quiet::<COLOR, ZOBRIST_OPTION>(played, piece);
    }

    #[inline(always)]
    fn apply_short_castle<const COLOR: bool, const ZOBRIST_OPTION: bool>(&mut self) {
        // we dont update the zobrist for short castle here becuase we already did that in update castling rights
        match COLOR {
            WHITE => {
                self.move_piece::<COLOR, ZOBRIST_OPTION>(Square::E1, Square::G1, BasePiece::King);
                self.move_piece::<COLOR, ZOBRIST_OPTION>(Square::H1, Square::F1, BasePiece::Rook);
            }
            BLACK => {
                self.move_piece::<COLOR, ZOBRIST_OPTION>(Square::E8, Square::G8, BasePiece::King);
                self.move_piece::<COLOR, ZOBRIST_OPTION>(Square::H8, Square::F8, BasePiece::Rook);
            }
        }
    }

    #[inline(always)]
    fn apply_long_castle<const COLOR: bool, const ZOBRIST_OPTION: bool>(&mut self) {
        match self.side_to_move {
            Color::White => {
                self.move_piece::<COLOR, ZOBRIST_OPTION>(Square::E1, Square::C1, BasePiece::King);
                self.move_piece::<COLOR, ZOBRIST_OPTION>(Square::A1, Square::D1, BasePiece::Rook);
            }
            Color::Black => {
                self.move_piece::<COLOR, ZOBRIST_OPTION>(Square::E8, Square::C8, BasePiece::King);
                self.move_piece::<COLOR, ZOBRIST_OPTION>(Square::A8, Square::D8, BasePiece::Rook);
            }
        }
    }

    #[inline(always)]
    fn apply_promotion<const COLOR: bool, const ZOBRIST_OPTION: bool>(&mut self, played: &MovePly) {
        self.remove_piece::<PAWN, COLOR, ZOBRIST_OPTION>(played.from());
        match played.flag().base_promotion_piece() {
            BasePiece::Knight => self.add_piece::<KNIGHT, COLOR, ZOBRIST_OPTION>(played.to()),
            BasePiece::Bishop => self.add_piece::<BISHOP, COLOR, ZOBRIST_OPTION>(played.to()),
            BasePiece::Rook => self.add_piece::<ROOK, COLOR, ZOBRIST_OPTION>(played.to()),
            BasePiece::Queen => self.add_piece::<QUEEN, COLOR, ZOBRIST_OPTION>(played.to()),

            _ => unreachable!(),
        }
    }

    #[inline(always)]
    fn apply_en_passant<const COLOR: bool, const ZOBRIST_OPTION: bool>(
        &mut self,
        played: &MovePly,
    ) {
        self.apply_quiet::<COLOR, ZOBRIST_OPTION>(played, BasePiece::Pawn);

        match COLOR {
            WHITE => self.remove_piece::<PAWN, BLACK, ZOBRIST_OPTION>(Square::from(
                self.en_passant_file as u8 + 32,
            )),
            BLACK => self.remove_piece::<PAWN, WHITE, ZOBRIST_OPTION>(Square::from(
                self.en_passant_file as u8 + 24,
            )),
        }
    }

    #[inline(always)]
    fn update_castling_rights<const ZOBRIST_OPTION: bool>(&mut self, from: Square, to: Square) {
        let previous_rights = self.castling_rights;

        self.castling_rights &= SQUARE_MOVED_CASTLING[from as usize];
        self.castling_rights &= SQUARE_MOVED_CASTLING[to as usize];

        if self.castling_rights == previous_rights {
            return;
        }

        if ZOBRIST_OPTION != INCREMENT_ZOBRIST {
            return;
        }

        if previous_rights & 0b0001 != 0 && self.castling_rights & 0b0001 == 0 {
            self.zobrist ^= ZOBRIST.short_castle(Color::White);
        }
        if previous_rights & 0b0010 != 0 && self.castling_rights & 0b0010 == 0 {
            self.zobrist ^= ZOBRIST.long_castle(Color::White);
        }

        if previous_rights & 0b0100 != 0 && self.castling_rights & 0b0100 == 0 {
            self.zobrist ^= ZOBRIST.short_castle(Color::Black);
        }
        if previous_rights & 0b1000 != 0 && self.castling_rights & 0b1000 == 0 {
            self.zobrist ^= ZOBRIST.long_castle(Color::Black);
        }
    }

    #[inline(always)]
    pub fn make_move<const COLOR: bool, const ZOBRIST_OPTION: bool>(
        &mut self,
        played: &MovePly,
        moving_piece: BasePiece,
    ) {
        let from = played.from();
        let to = played.to();

        if ZOBRIST_OPTION == INCREMENT_ZOBRIST && self.can_en_passant {
            self.zobrist ^= ZOBRIST.pawn_jump(self.en_passant_file);
        }
        self.can_en_passant = false;

        let enemy_indicies = match COLOR {
            WHITE => NUM_BASE_PIECES..NUM_PIECES,
            BLACK => 0..NUM_BASE_PIECES,
        };

        let new_enemy_occupancy = self.occupancy_them_const::<COLOR>() & !to.mask();

        for bitboard_index in enemy_indicies {
            if ZOBRIST_OPTION == INCREMENT_ZOBRIST {
                if self.bitboards[bitboard_index] & to.mask() != 0 {
                    self.bitboards[bitboard_index] ^= to.mask();
                    self.zobrist ^= ZOBRIST.square_zobrist((bitboard_index as u8).into(), to);
                    break;
                }
            } else {
                self.bitboards[bitboard_index] &= new_enemy_occupancy;
            }
        }

        match COLOR {
            WHITE => self.black_occupancy = new_enemy_occupancy,
            BLACK => self.white_occupancy = new_enemy_occupancy,
        }

        let is_capture = self.occupancy_them_const::<COLOR>() & to.mask() != 0;
        if ZOBRIST_OPTION == INCREMENT_ZOBRIST
            && (matches!(moving_piece, BasePiece::Pawn) || is_capture)
        {
            self.half_move_clock = 0;
        } else {
            self.half_move_clock += 1;
        }

        self.update_castling_rights::<ZOBRIST_OPTION>(from, to);

        match played.flag() {
            MoveFlag::None => self.apply_quiet::<COLOR, ZOBRIST_OPTION>(played, moving_piece),
            MoveFlag::DoubleJump => {
                self.apply_double_jump::<COLOR, ZOBRIST_OPTION>(played, moving_piece)
            }
            MoveFlag::CastleShort => self.apply_short_castle::<COLOR, ZOBRIST_OPTION>(),
            MoveFlag::CastleLong => self.apply_long_castle::<COLOR, ZOBRIST_OPTION>(),
            MoveFlag::EnPassantCapture => self.apply_en_passant::<COLOR, ZOBRIST_OPTION>(played),
            _ => self.apply_promotion::<COLOR, ZOBRIST_OPTION>(played),
        }

        self.occupancy = self.white_occupancy | self.black_occupancy;
        self.side_to_move = !self.side_to_move;

        if ZOBRIST_OPTION == INCREMENT_ZOBRIST {
            self.zobrist ^= ZOBRIST.side_to_move();
        }
    }

    /* pub fn make_null_move(&mut self) {
        self.push_board_state(MovePly::default(), NoPiece);
        self.zobrist ^= ZOBRIST.side_to_move();
        self.side_to_move = !self.side_to_move;
        self.can_en_passant = false;
    } */

    /* pub fn undo_null_move(&mut self) {
        let last_board_state = self.board_states[self.cur_board_state - 1];

        self.castling_rights = last_board_state.castling_rights;
        self.en_passant_file = last_board_state.en_passant_file;
        self.can_en_passant = last_board_state.can_en_passant;
        self.half_move_clock = last_board_state.half_move_clock;
        self.zobrist = last_board_state.zobrist;
        self.in_check = last_board_state.in_check;

        self.cur_board_state -= 1;
        self.side_to_move = !self.side_to_move;
    } */
}

const TOP_SECTION: &str = "    ┌─────┬─────┬─────┬─────┬─────┬─────┬─────┬─────┐";
const MIDDLE_SECTION: &str = "    ├─────┼─────┼─────┼─────┼─────┼─────┼─────┼─────┤";
const BOTTOM_SECTION: &str = "    └─────┴─────┴─────┴─────┴─────┴─────┴─────┴─────┘";
const FILE_LABEL: &str = "       a     b     c     d     e     f     g     h   ";
const SIDE_BAR: &str = "│";

impl Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut pretty_print = TOP_SECTION.to_string();

        for i in 0..NUM_SQUARES {
            if i % 8 == 0 {
                if i != 0 {
                    pretty_print += &(SIDE_BAR.to_owned() + "\n" + MIDDLE_SECTION + "\n");
                } else {
                    pretty_print += "\n";
                }

                pretty_print += &(" ".to_owned() + &((i ^ 56) / 8 + 1).to_string() + "  ");
            }

            let square = Square::from((i ^ 56) as u8);
            let piece = self.piece_at(square);
            let piece_as_str = piece.to_string();

            pretty_print += &(SIDE_BAR.to_owned() + "  " + piece_as_str.as_str() + "  ");
        }

        pretty_print += &(SIDE_BAR.to_owned() + "\n" + BOTTOM_SECTION + "\n" + FILE_LABEL + "\n");

        write!(f, "{}", pretty_print)
    }
}
