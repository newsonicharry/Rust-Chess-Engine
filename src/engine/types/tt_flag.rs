#[repr(u8)]
#[derive(Clone, Copy)]
pub enum TTFlag{
    Upper,
    Lower,
    Exact,
}

impl From<u8> for TTFlag{
    fn from(val: u8) -> Self{
        unsafe{std::mem::transmute(val)}
    }
}

