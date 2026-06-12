#[repr(u8)]
#[derive(Clone, Copy, Default)]
pub enum TTFlag {
    #[default]
    Upper,
    Lower,
    Exact,
}

impl From<u8> for TTFlag {
    fn from(val: u8) -> Self {
        unsafe { std::mem::transmute(val) }
    }
}
