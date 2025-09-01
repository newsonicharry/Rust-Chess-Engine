use crate::chess::bitboard::Bitboard;
use crate::chess::board_state::BoardState;
use crate::chess::consts::{MAX_MOVES, NUM_PIECES, NUM_SQUARES};
use crate::chess::move_ply::MovePly;
use crate::chess::piece_list::PieceList;
use crate::chess::types::color::Color;
use crate::chess::types::file::File;
use crate::chess::types::move_flag::MoveFlag;
use crate::chess::types::piece::Piece::{BlackBishop, BlackKing, BlackPawn, BlackQueen, BlackRook, NoPiece, WhiteBishop, WhiteKing, WhitePawn, WhiteQueen, WhiteRook};
use crate::chess::types::piece::{char_to_piece, BasePiece, Piece, ITER_BLACK, ITER_WHITE};
use crate::chess::types::rank::Rank;
use crate::chess::types::square::Square;
use std::fmt::Display;

// if a piece on a certain square moves then the castling rights must change as well
const SQUARE_MOVED_CASTLING: [u8; NUM_SQUARES] = [13, 15, 15, 15, 12, 15, 15, 14, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 7, 15, 15, 15, 3, 15, 15, 11];

pub struct Board{
    bitboards: [Bitboard; NUM_PIECES],
    piece_lists: [PieceList; NUM_PIECES],
    piece_squares: [Piece; NUM_SQUARES],

    side_to_move: Color,

    white_occupancy: u64,
    black_occupancy: u64,
    occupancy: u64,

    en_passant_file: File,
    can_en_passant: bool,
    castling_rights: u8,
    half_move_clock: u8,
    zobrist: u64,

    board_states: [BoardState; MAX_MOVES],
    cur_board_state: usize
}


impl Default for Board {
    fn default() -> Board {
        let board = Board{
            bitboards: [Bitboard::default(); NUM_PIECES],
            piece_lists: [PieceList::default(); NUM_PIECES],
            piece_squares: [NoPiece; NUM_SQUARES],

            side_to_move: Color::White,

            white_occupancy: 0,
            black_occupancy: 0,
            occupancy: 0,

            en_passant_file: File::default(),
            can_en_passant: false,
            castling_rights: 0,
            half_move_clock: 0,
            zobrist: 0,

            board_states: [BoardState::default(); MAX_MOVES],
            cur_board_state: 0,
        };

        board
    }
}


impl Board{

    // fen string is considered accurate at this point
    // determining the fen strings accuracy is the job of the uci
    pub fn new(&mut self, fen: &str){
        let split_fen: Vec<&str> =  fen.split_whitespace().collect();

        let fen_sequence = split_fen.get(0).unwrap();
        let ranks= fen_sequence.split('/').collect::<Vec<&str>>();

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

                self.add_piece(piece, cur_square);
            }
        }
        let side_to_move_str = split_fen.get(1).unwrap();
        if side_to_move_str.contains("b") { self.side_to_move = Color::Black; }

        let castling_rights_str = split_fen.get(2).unwrap();
        if castling_rights_str.contains("K") { self.castling_rights |= 0b0001 }
        if castling_rights_str.contains("Q") { self.castling_rights |= 0b0010 }
        if castling_rights_str.contains("k") { self.castling_rights |= 0b0100 }
        if castling_rights_str.contains("q") { self.castling_rights |= 0b1000 }


        self.update_occupancy();
    }

    pub fn color_orthogonal_bitboard(&self, color: Color) -> u64{
        match color {
            Color::White => self.bitboards[WhiteRook as usize].0 | self.bitboards[WhiteQueen as usize].0,
            Color::Black => self.bitboards[BlackRook as usize].0 | self.bitboards[BlackQueen as usize].0,
        }
    }

    pub fn color_diagonal_bitboard(&self, color: Color) -> u64{
        match color {
            Color::White => self.bitboards[WhiteBishop as usize].0 | self.bitboards[WhiteQueen as usize].0,
            Color::Black => self.bitboards[BlackBishop as usize].0 | self.bitboards[BlackQueen as usize].0,
        }
    }

    pub fn color_bitboard(&self, piece: BasePiece, color: Color) -> u64{
        match color {
            Color::White => self.bitboards[piece as usize].0,
            Color::Black => self.bitboards[piece as usize + 6].0,
        }
    }

    pub fn bitboard(&self, piece: Piece) -> u64{
        self.bitboards[piece as usize].0
    }

    pub fn pieces_of(&self, piece: Piece) -> &[Square] {
        &self.piece_lists[piece as usize].indexes()
    }

    pub fn color_pieces_of(&self, piece: BasePiece, color: Color) -> &[Square] {
        match color {
            Color::White => &self.piece_lists[piece as usize].indexes(),
            Color::Black => &self.piece_lists[piece as usize + 6].indexes(),
        }
    }

    pub fn piece_at(&self, square: Square) -> Piece{
        self.piece_squares[square as usize]
    }

    pub fn king_square(&self, color: Color) -> Square{

        match color {
            Color::White => self.piece_lists[WhiteKing as usize].indexes()[0],
            Color::Black => self.piece_lists[BlackKing as usize].indexes()[0],
        }
    }

    pub fn color_occupancy(&self, color: Color) -> u64{
        match color {
            Color::White => self.white_occupancy,
            Color::Black => self.black_occupancy,
        }
    }

    pub fn all_occupancy(&self) -> u64{
        self.occupancy
    }

    pub fn side_to_move(&self) -> Color{
        self.side_to_move
    }

    pub fn en_passant_file(&self) -> Option<File>{
        match self.can_en_passant {
            true => Some(self.en_passant_file),
            false => None,
        }
    }

    pub fn has_short_castle_rights(&self, color: Color) -> bool{
        match color {
            Color::White => self.castling_rights & 0b0001 != 0,
            Color::Black => self.castling_rights & 0b0100 != 0,
        }
    }

    pub fn has_long_castle_rights(&self, color: Color) -> bool{
        match color {
            Color::White => self.castling_rights & 0b0010 != 0,
            Color::Black => self.castling_rights & 0b1000 != 0,
        }
    }

    fn update_occupancy(&mut self){
        self.white_occupancy = 0;
        self.black_occupancy = 0;
        self.occupancy = 0;

        for piece in Piece::iterator::<ITER_WHITE>(){
            let bitboard =  self.bitboards[piece as usize].0;
            self.white_occupancy |= bitboard;
            self.occupancy |= bitboard;
        }

        for piece in Piece::iterator::<ITER_BLACK>(){
            let bitboard =  self.bitboards[piece as usize].0;
            self.black_occupancy |= bitboard;
            self.occupancy |= bitboard;
        }
    }


    fn push_board_state(&mut self, played: MovePly, captured: Piece){
        self.board_states[self.cur_board_state].played = played;
        self.board_states[self.cur_board_state].captured = captured;
        self.board_states[self.cur_board_state].half_move_clock = self.half_move_clock;
        self.board_states[self.cur_board_state].castling_rights = self.castling_rights;
        self.board_states[self.cur_board_state].en_passant_file = self.en_passant_file;
        self.board_states[self.cur_board_state].can_en_passant = self.can_en_passant;

        self.cur_board_state += 1;
    }
    #[inline(always)]
    fn add_piece(&mut self, piece: Piece, square: Square){
        self.bitboards[piece as usize].add_piece(square);
        self.piece_lists[piece as usize].add_piece(square);
        self.piece_squares[square as usize] = piece;
    }
    #[inline(always)]
    fn remove_piece(&mut self, piece: Piece, square: Square){
        self.bitboards[piece as usize].remove_piece(square);
        self.piece_lists[piece as usize].remove_piece(square);
        self.piece_squares[square as usize] = NoPiece;
    }
    #[inline(always)]
    fn move_piece(&mut self, piece: Piece, from: Square, to: Square){
        self.bitboards[piece as usize].move_piece(from, to);
        self.piece_lists[piece as usize].move_piece(from, to);

        self.piece_squares[from as usize] = NoPiece;
        self.piece_squares[to as usize] = piece;
    }
    fn apply_quiet(&mut self, played: MovePly){
        let from = played.from();
        self.move_piece(self.piece_at(from), from, played.to())
    }
    fn apply_double_jump(&mut self, played: MovePly){
        self.en_passant_file =  played.from().file();
        self.can_en_passant = true;
        self.apply_quiet(played);
    }
    fn apply_kingside_castle(&mut self){
        match self.side_to_move {
            Color::White => {
                self.move_piece(WhiteKing, Square::E1, Square::G1);
                self.move_piece(WhiteRook, Square::H1, Square::F1);
            }
            Color::Black => {
                self.move_piece(BlackKing, Square::E8, Square::G8);
                self.move_piece(BlackRook, Square::H8, Square::F8);
            }
        }
    }
    fn apply_queenside_castle(&mut self){
        match self.side_to_move {
            Color::White => {
                self.move_piece(WhiteKing, Square::E1, Square::C1);
                self.move_piece(WhiteRook, Square::A1, Square::D1);
            }
            Color::Black => {
                self.move_piece(BlackKing, Square::E8, Square::C8);
                self.move_piece(BlackRook, Square::A8, Square::D8);
            }
        }
    }

    fn apply_promotion(&mut self, played: MovePly){
        self.remove_piece(self.piece_at(played.from()), played.from());
        self.add_piece(played.flag().promotion_piece(self.side_to_move), played.to());

    }

    fn apply_en_passant(&mut self, played: MovePly){
        self.apply_quiet(played);

        match self.side_to_move{
            Color::White => self.remove_piece(BlackPawn, Square::from(self.en_passant_file as u8 + 32)),
            Color::Black => self.remove_piece(WhitePawn, Square::from(self.en_passant_file as u8 + 24)),
        }
    }

    fn reverse_quiet(&mut self, played: MovePly){
        let to = played.to();
        self.move_piece(self.piece_at(to), to, played.from())
    }

    fn reverse_kingside_castle(&mut self){
        match self.side_to_move {
            Color::White => {
                self.move_piece(WhiteKing, Square::G1, Square::E1);
                self.move_piece(WhiteRook, Square::F1, Square::H1);
            }
            Color::Black => {
                self.move_piece(BlackKing, Square::G8, Square::E8);
                self.move_piece(BlackRook, Square::F8, Square::H8);
            }
        }
    }
    fn reverse_queenside_castle(&mut self){
        match self.side_to_move {
            Color::White => {
                self.move_piece(WhiteKing, Square::C1, Square::E1);
                self.move_piece(WhiteRook, Square::D1, Square::A1);
            }
            Color::Black => {
                self.move_piece(BlackKing, Square::C8, Square::E8);
                self.move_piece(BlackRook, Square::D8, Square::A8);
            }
        }
    }

    fn reverse_promotion(&mut self, played: MovePly){
        self.remove_piece(self.piece_at(played.to()), played.to());

        let original_pawn = match self.side_to_move {
            Color::White => WhitePawn,
            Color::Black => BlackPawn,
        };

        self.add_piece(original_pawn, played.from());
    }

    fn reverse_en_passant(&mut self, played: MovePly){
        self.reverse_quiet(played);

        match self.side_to_move{
            Color::White => self.add_piece(BlackPawn, Square::from(self.en_passant_file as u8 + 32)),
            Color::Black => self.add_piece(WhitePawn, Square::from(self.en_passant_file as u8 + 24)),
        }
    }


    pub fn make_move(&mut self, played: MovePly){

        let from = played.from();
        let to = played.to();
        let capture = self.piece_at(to);

        self.push_board_state(played, capture);

        self.can_en_passant = false;
        if capture.is_piece() {
            self.remove_piece(capture, to);
        }

        self.castling_rights &= SQUARE_MOVED_CASTLING[from as usize];
        self.castling_rights &= SQUARE_MOVED_CASTLING[to as usize];

        match played.flag() {
            MoveFlag::None => self.apply_quiet(played),
            MoveFlag::DoubleJump => self.apply_double_jump(played),
            MoveFlag::CastleKingSide => self.apply_kingside_castle(),
            MoveFlag::CastleQueenSide => self.apply_queenside_castle(),
            MoveFlag::EnPassantCapture => self.apply_en_passant(played),
            _ => self.apply_promotion(played),
        }

        self.side_to_move = !self.side_to_move;
        self.update_occupancy()
    }

    pub fn undo_move(&mut self){
        let last_board_state = unsafe { self.board_states[self.cur_board_state-1] };
        let last_played = last_board_state.played;

        self.castling_rights = last_board_state.castling_rights;
        self.en_passant_file = last_board_state.en_passant_file;
        self.can_en_passant = last_board_state.can_en_passant;
        self.half_move_clock = last_board_state.half_move_clock;

        self.side_to_move = !self.side_to_move;


        match last_played.flag() {
            MoveFlag::None | MoveFlag::DoubleJump => self.reverse_quiet(last_played),
            MoveFlag::CastleKingSide => self.reverse_kingside_castle(),
            MoveFlag::CastleQueenSide => self.reverse_queenside_castle(),
            MoveFlag::EnPassantCapture => self.reverse_en_passant(last_played),
            _ => self.reverse_promotion(last_played),
        }


        if last_board_state.captured.is_piece() && !last_played.flag().is_en_passant_capture(){
            self.add_piece(last_board_state.captured, last_played.to());
        }

        self.cur_board_state -= 1;
        self.update_occupancy();
    }



}


const TOP_SECTION: &str    = "    ┌─────┬─────┬─────┬─────┬─────┬─────┬─────┬─────┐";
const MIDDLE_SECTION: &str = "    ├─────┼─────┼─────┼─────┼─────┼─────┼─────┼─────┤";
const BOTTOM_SECTION: &str = "    └─────┴─────┴─────┴─────┴─────┴─────┴─────┴─────┘";
const FILE_LABEL: &str =     "       a     b     c     d     e     f     g     h   ";
const SIDE_BAR: &str = "│";


impl Display for Board {

    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {

        let mut pretty_print = TOP_SECTION.to_string();

        for i in 0..NUM_SQUARES {

            if i % 8 == 0 {

                if i != 0 {
                    pretty_print += &*(SIDE_BAR.to_owned() + "\n" + MIDDLE_SECTION + "\n");
                }
                else {
                    pretty_print += "\n";
                }

                pretty_print += &*(" ".to_owned() + &*((i ^ 56) / 8 + 1).to_string() + "  ");
            }

            let square = Square::from((i ^ 56) as u8);
            let piece = self.piece_at(square);
            let piece_as_str = piece.to_string();

            pretty_print += &*(SIDE_BAR.to_owned() + "  " + piece_as_str.as_str() + "  ");

        }

        pretty_print += &*(SIDE_BAR.to_owned() + "\n" + BOTTOM_SECTION + "\n" + FILE_LABEL + "\n");

        write!(f, "{}", pretty_print)


    }
}