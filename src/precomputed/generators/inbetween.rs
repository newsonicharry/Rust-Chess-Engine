use crate::chess::types::file::File;
use crate::chess::types::rank::Rank;
use crate::chess::types::square::Square;
use crate::chess::consts::NUM_SQUARES;

pub struct InBetween{
    pub in_between: [[u64; NUM_SQUARES]; NUM_SQUARES]
}
impl InBetween {

    pub fn new() -> InBetween {
        let mut in_between = InBetween{
            in_between: [[0; NUM_SQUARES]; NUM_SQUARES]
        };

        in_between.create_in_between();

        in_between

    }
    fn create_in_between(&mut self){
        for i in 0..NUM_SQUARES {
            let from_square = Square::from(i as u8);

            for j in 0..NUM_SQUARES {
                let to_square = Square::from(j as u8);

                let start_x = from_square.file() as u8;
                let target_x = to_square.file() as u8;

                let start_y = from_square.rank() as u8;
                let target_y = to_square.rank() as u8;

                let mut dir: (i8, i8) = (0 , 0);



                // could combine these but im trying to avoid a criminal amount of nesting
                // orthogonal
                if (start_x == target_x) && (start_y < target_y){ dir = (0, 1); }
                if (start_x == target_x) && (start_y > target_y){ dir = (0, -1); }
                if (start_y == target_y) && (start_x < target_x){ dir = (1, 0); }
                if (start_y == target_y) && (start_x > target_x){ dir = (-1, 0); }


                let dif_x =  target_x as i8 - start_x as i8;
                let dif_y = target_y as i8 - start_y as i8;
                // is a diagonal if the x and y parts of the triangle are the same (45 45 90 triangle)
                let is_diagonal = dif_x.abs() == dif_y.abs();

                // diagonal
                if is_diagonal && dif_x.is_positive() && dif_y.is_positive() { dir = (1, 1); }
                if is_diagonal && dif_x.is_negative() && dif_y.is_positive() { dir = (-1, 1); }
                if is_diagonal && dif_x.is_positive() && dif_y.is_negative() { dir = (1, -1); }
                if is_diagonal && dif_x.is_negative() && dif_y.is_negative() { dir = (-1, -1); }


                if dir == (0, 0) { continue; }

                // finally a use case for a do while loop and rust hasn't implemented it...
                // (i've been waiting 6 years to finally find a real place to use a do while too)
                let mut cur_x = (start_x as i8 + dir.0) as u8;
                let mut cur_y = (start_y as i8 + dir.1) as u8;
                while (cur_x != target_x) || (cur_y != target_y) {

                    let file = File::from(cur_x);
                    let rank = Rank::from(cur_y);

                    let new_square = Square::from((file, rank));

                    self.in_between[i][j] |= new_square.mask();

                    cur_x = (cur_x as i8 + dir.0) as u8;
                    cur_y = (cur_y as i8 + dir.1) as u8;


                }


            }
        }

    }
}