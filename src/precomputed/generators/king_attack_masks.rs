use crate::{
    chess::{
        consts::{NUM_BASE_PIECES, NUM_SQUARES},
        move_generator::{
            BLACK_LONG_NOT_ATTACKED_SQUARES, BLACK_SHORT_NOT_ATTACKED_SQUARES,
            WHITE_LONG_NOT_ATTACKED_SQUARES, WHITE_SHORT_NOT_ATTACKED_SQUARES,
        },
        types::{color::Color, piece::BasePiece, square::Square},
    },
    precomputed::{
        accessor::{MOVEMENT_MASKS, bishop_lookup, queen_lookup, rook_lookup},
        generators::helpers::generate_blockers,
    },
};

pub struct KingAttackMasks {
    knight: Vec<u64>,
    bishop: Vec<u64>,
    rook: Vec<u64>,
    queen: Vec<u64>,
    king: Vec<u64>,

    white_pawns: Vec<u64>,
    black_pawns: Vec<u64>,

    white_long_castle: [u64; NUM_BASE_PIECES],
    white_short_castle: [u64; NUM_BASE_PIECES],
    black_long_castle: [u64; NUM_BASE_PIECES],
    black_short_castle: [u64; NUM_BASE_PIECES],

    offsets: [usize; NUM_SQUARES],
}

impl KingAttackMasks {
    pub fn new() -> Self {
        let mut lookup = Self {
            knight: Vec::new(),
            bishop: Vec::new(),
            rook: Vec::new(),
            queen: Vec::new(),
            king: Vec::new(),

            white_pawns: Vec::new(),
            black_pawns: Vec::new(),

            white_long_castle: [0; NUM_BASE_PIECES],
            white_short_castle: [0; NUM_BASE_PIECES],
            black_long_castle: [0; NUM_BASE_PIECES],
            black_short_castle: [0; NUM_BASE_PIECES],

            offsets: [0; NUM_SQUARES],
        };

        lookup.generate_king_masks();
        lookup.generate_castling_masks();

        lookup
    }

    fn reserve_capacity(&mut self, amount: usize) {
        let new_entries = vec![0; amount];

        self.knight.extend(&new_entries);
        self.rook.extend(&new_entries);
        self.bishop.extend(&new_entries);
        self.queen.extend(&new_entries);
        self.king.extend(&new_entries);

        self.white_pawns.extend(&new_entries);
        self.black_pawns.extend(&new_entries);
    }

    fn generate_castling_masks(&mut self) {
        macro_rules! generate_castling_mask {
            ($not_attacked_squares: ident, $lookup: expr, $pawn_color: expr) => {{
                let mut mask = $not_attacked_squares;
                while mask != 0 {
                    let lsb = mask.trailing_zeros();

                    $lookup[BasePiece::Pawn as usize] |=
                        MOVEMENT_MASKS.pawn_attacks($pawn_color, lsb.into());
                    $lookup[BasePiece::Knight as usize] |= MOVEMENT_MASKS.knight[lsb as usize];
                    $lookup[BasePiece::King as usize] |= MOVEMENT_MASKS.king[lsb as usize];
                    $lookup[BasePiece::Bishop as usize] |= bishop_lookup(lsb.into(), 0);
                    $lookup[BasePiece::Rook as usize] |= rook_lookup(lsb.into(), 0);
                    $lookup[BasePiece::Queen as usize] |= queen_lookup(lsb.into(), 0);

                    mask &= mask - 1;
                }
            }};
        }

        generate_castling_mask!(
            WHITE_SHORT_NOT_ATTACKED_SQUARES,
            self.white_short_castle,
            Color::White
        );

        generate_castling_mask!(
            BLACK_SHORT_NOT_ATTACKED_SQUARES,
            self.black_short_castle,
            Color::Black
        );
        generate_castling_mask!(
            WHITE_LONG_NOT_ATTACKED_SQUARES,
            self.white_long_castle,
            Color::White
        );
        generate_castling_mask!(
            BLACK_LONG_NOT_ATTACKED_SQUARES,
            self.black_long_castle,
            Color::Black
        );
    }

    fn generate_king_masks(&mut self) {
        let mut last_offset: usize = 0;
        for i in 0..NUM_SQUARES {
            let king_move_mask = MOVEMENT_MASKS.king[i];
            let blockers = generate_blockers(king_move_mask);
            self.reserve_capacity(blockers.len());

            for blocker in &blockers {
                let mut new_mask = king_move_mask & !blocker;
                while new_mask > 0 {
                    let lsb = new_mask.trailing_zeros();

                    let pext = unsafe { std::arch::x86_64::_pext_u64(*blocker, king_move_mask) };
                    let key = pext as usize + last_offset;

                    self.knight[key] |= MOVEMENT_MASKS.knight[lsb as usize];
                    self.king[key] |=
                        MOVEMENT_MASKS.king[lsb as usize] & !king_move_mask & !(1 << i);
                    self.bishop[key] |= bishop_lookup(lsb.into(), 0);
                    self.rook[key] |= rook_lookup(lsb.into(), 0);
                    self.queen[key] |= queen_lookup(lsb.into(), 0);

                    self.white_pawns[key] |= MOVEMENT_MASKS.pawn_attacks(Color::White, lsb.into());
                    self.black_pawns[key] |= MOVEMENT_MASKS.pawn_attacks(Color::Black, lsb.into());

                    new_mask &= new_mask - 1;
                }
            }

            self.offsets[i] = last_offset;
            last_offset += blockers.len();
        }
    }

    fn lookup<const PIECE: u8>(
        &self,
        king_square: Square,
        king_mask_occupancy: u64,
        color: Color,
    ) -> u64 {
        let pext = unsafe {
            std::arch::x86_64::_pext_u64(
                king_mask_occupancy,
                MOVEMENT_MASKS.king[king_square as usize],
            )
        };

        let key = self.offsets[king_square as usize] + pext as usize;

        match BasePiece::from(PIECE) {
            BasePiece::Pawn => match color {
                Color::White => self.white_pawns[key],
                Color::Black => self.black_pawns[key],
            },
            BasePiece::Knight => self.knight[key],
            BasePiece::Bishop => self.bishop[key],
            BasePiece::Rook => self.rook[key],
            BasePiece::Queen => self.queen[key],
            BasePiece::King => self.king[key],
        }
    }
}

pub struct KingAttackLookupData {
    pub king_square: Square,
    pub king_mask_occupancy: u64,
    pub side_to_move: Color,
    pub potential_short_castle: bool,
    pub potential_long_castle: bool,
}

macro_rules! create_lookup {
    ($name: tt, $piece: expr) => {
        impl KingAttackMasks {
            pub fn $name(&self, lookup_data: &KingAttackLookupData) -> u64 {
                let mut base_mask = self.lookup::<$piece>(
                    lookup_data.king_square,
                    lookup_data.king_mask_occupancy,
                    lookup_data.side_to_move,
                );

                if lookup_data.potential_short_castle {
                    base_mask |= match lookup_data.side_to_move {
                        Color::White => self.white_short_castle[$piece as usize],
                        Color::Black => self.black_short_castle[$piece as usize],
                    };
                }

                if lookup_data.potential_long_castle {
                    base_mask |= match lookup_data.side_to_move {
                        Color::White => self.white_long_castle[$piece as usize],
                        Color::Black => self.black_long_castle[$piece as usize],
                    };
                }

                base_mask
            }
        }
    };
}

create_lookup!(pawn_lookup, { BasePiece::Pawn as u8 });
create_lookup!(knight_lookup, { BasePiece::Knight as u8 });
create_lookup!(bishop_lookup, { BasePiece::Bishop as u8 });
create_lookup!(rook_lookup, { BasePiece::Rook as u8 });
create_lookup!(queen_lookup, { BasePiece::Queen as u8 });
create_lookup!(king_lookup, { BasePiece::King as u8 });
