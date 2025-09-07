
// OPTION_NAME, RANGE, DEFAULT
pub const SPIN_OPTION_TABLE: &[(&str, &str, &str)] = &[
    ("Hash", "1-4096", "16"),
    ("Threads", "1-1024", "1")
];

pub const BUTTON_OPTION_TABLE: &[&str] = &[
    "Clear Hash",
];


pub enum OptionType{
    Spin,
    Button,
    NoType,
}

impl PartialEq<OptionType> for OptionType {
    fn eq(&self, other: &OptionType) -> bool {
        other == self
    }
}