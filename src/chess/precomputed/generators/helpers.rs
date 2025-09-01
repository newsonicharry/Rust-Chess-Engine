use crate::chess::types::file::File;
use crate::chess::types::rank::Rank;
use crate::chess::types::square::Square;

pub const NO_EDGE: bool = false;
pub const INCLUDE_EDGE: bool = true;


// using a tuple of directions from the origin, it makes a movement mask for a given piece position
// useful for pieces such as the knight, king and pawn with constant movements (unlike sliding pieces)
pub fn create_static_mask(directions: &[(i8, i8)], square: Square) -> u64{
    let mut mask: u64 = 0;
    
    let x_cord = square.file() as u8;
    let y_cord = square.rank() as u8;

    for direction in directions {
        let x_dir = direction.0;
        let y_dir = direction.1;

        let new_x = x_cord as i8 + x_dir;
        let new_y = y_cord as i8 + y_dir;

        if (new_x >  7 || new_x < 0) || (new_y >  7 || new_y < 0) {continue;}

        let file = File::from(new_x as u8);
        let rank = Rank::from(new_y as u8);

        let new_square = Square::from((file, rank));

        mask |= new_square.mask();

    }

    mask
}

// meant for sliding pieces in which the direction is looped over until the end of the board is reached
pub fn create_dynamic_mask<const EDGE: bool>(directions: &[(i8, i8)], square: Square) -> u64{
    
    let mut mask: u64 = 0;

    for direction in directions {
        let x_dir = direction.0;
        let y_dir = direction.1;
        
        let mut x_cord = square.file() as i8;
        let mut y_cord = square.rank() as i8;

        loop {
            let new_x = x_cord + x_dir;
            let new_y = y_cord + y_dir;

            if (new_x >  7 || new_x < 0) || (new_y > 7 || new_y < 0) {
                break;
            }

            if (EDGE == NO_EDGE) && ((x_dir != 0 && (new_x > 6 || new_x < 1)) || (y_dir != 0 && (new_y > 6 || new_y < 1))) {

                break;
            }


            let file = File::from(new_x as u8);
            let rank = Rank::from(new_y as u8);
            
            let new_square = Square::from((file, rank));

            mask |= new_square.mask();

            x_cord = new_x;
            y_cord = new_y

        }


    }

    mask

}