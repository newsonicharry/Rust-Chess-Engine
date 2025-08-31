use rand::Rng;
use crate::chess::consts::{NUM_SQUARES, NUM_FILES};
pub struct Zobrist{
    pub squares: [u64; NUM_SQUARES],
    pub en_passant: [u64; NUM_FILES],
    pub castling_rights: [u64; 4],
    pub side_to_move: u64,
}

impl Zobrist{
    pub fn new() -> Self{
        let mut zobrist = Zobrist{
            squares: [0; NUM_SQUARES],
            en_passant: [0; NUM_FILES],
            castling_rights: [0; 4],
            side_to_move: 0,
        };

        for i in 0..NUM_SQUARES  { zobrist.squares[i] = Self::gen_rand(); }
        for i in 0..NUM_FILES { zobrist.en_passant[i] = Self::gen_rand(); }
        for i in 0..4     { zobrist.castling_rights[i] = Self::gen_rand(); }

        zobrist.side_to_move = Self::gen_rand();

        zobrist


    }

    fn gen_rand() -> u64{
        rand::thread_rng().r#gen::<u64>()
    }
}