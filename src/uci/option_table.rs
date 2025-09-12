
// OPTION_NAME, MIN, MAX, DEFAULT
pub const SPIN_OPTION_TABLE: &[(&str, u16, u16, u16)] = &[
    ("Hash", 1, 32768, 16),
    ("Threads", 1, 1024, 1)
];

pub const BUTTON_OPTION_TABLE: &[&str] = &[
    "Clear Hash",
];


pub fn print_option_table(){

    for (name, min, max, default) in SPIN_OPTION_TABLE {
        println!("option name {name} type spin default {default} min {min} max {max}",)
    }

    for name in BUTTON_OPTION_TABLE {
        println!("option name {name} type button")
    }
}


