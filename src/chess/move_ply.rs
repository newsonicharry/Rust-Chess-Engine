use crate::chess::types::move_flag::MoveFlag;
use crate::chess::types::square::Square;


const SQUARE_MASK: u16 = 0b111111;
const TO_SHIFT: u8 = 6;
const FLAG_SHIFT: u8 = 12;

#[derive(Clone, Copy)]
pub struct MovePly{
    //   flag   to    from
    // 0b1111 111111 111111
    packed_data: u16
}

impl Default for MovePly{
    fn default() -> Self{
        MovePly{packed_data: 0}
    }
}

impl MovePly{
    pub fn new(from: Square, to: Square, flag: MoveFlag) -> Self{
        let packed_data =  ((flag as u16) << FLAG_SHIFT)
                         | ((to as u16) << TO_SHIFT)
                         | from as u16;

        MovePly{ packed_data }
    }


    pub fn from(&self) -> Square{
        Square::from((self.packed_data & SQUARE_MASK) as u8)
    }

    pub fn to(&self) -> Square{
        Square::from(((self.packed_data >> TO_SHIFT) & SQUARE_MASK) as u8)
    }

    pub fn flag(&self) -> MoveFlag{
        MoveFlag::from((self.packed_data >> FLAG_SHIFT) as u8)
    }
}