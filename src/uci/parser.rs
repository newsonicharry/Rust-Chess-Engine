use std::str::FromStr;
use crate::uci::commands::UCICommands;
use crate::uci::option_table::{SPIN_OPTION_TABLE, BUTTON_OPTION_TABLE, OptionType};
pub struct UCIParser {}

const START_POS: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

impl UCIParser {


    fn collect_until_end_or_breakpoint(start_index: usize, split_messages: &Vec<&str>, break_point: Option<&str>) -> Result<String, ()> {
        let mut final_str = "".to_string();

        for i in start_index..split_messages.len() {
            let section = *split_messages.get(i).unwrap();

            if let Some(break_point) = break_point {
                if section == break_point {
                    break
                }
            }


            final_str.push_str(section);
        }

        if final_str == "" {
            return Err(())
        }

        Ok(final_str.to_string())
    }

    fn parse_set_option(split_message: Vec<&str>) -> UCICommands {

        let original_message = String::from(split_message.join(" "));

        match split_message.get(1) {
            Some(msg) => if *msg != "name" {
                println!("Command setoption did not include 'name' as a parameter.");
                return UCICommands::Unknown(original_message);
            }
            None => {
                println!("Command setoption must be formatted as 'setoption name <option-name>'");
                return UCICommands::Unknown(original_message);
            },
        }

        let option_name_wrapped = Self::collect_until_end_or_breakpoint(1, &split_message, Some("name"));

        if option_name_wrapped.is_err() {
            println!("Command setoption did not include the value for name");
            return UCICommands::Unknown(original_message);
        }

        let option_name = option_name_wrapped.unwrap();
        let mut option_type = OptionType::NoType;

        for item in SPIN_OPTION_TABLE.iter() {
            if option_name == item.0 { option_type = OptionType::Spin }
        }

        for item in BUTTON_OPTION_TABLE.iter() {
            if option_name == *item { option_type = OptionType::Button }
        }


        if option_type == OptionType::NoType {
            println!("Command setoption of name '{option_name}' is not a valid option.");
            return UCICommands::Unknown(original_message);
        }

        if option_type == OptionType::Spin {
            if !split_message.contains(&"value") {
                println!("Command setoption of option type spin must include a value parameter.");
                return UCICommands::Unknown(original_message);
            }

            let start_index = split_message.iter().position(|&x| x == "value").unwrap()+1;

            let value_wrapped = Self::collect_until_end_or_breakpoint(start_index, &split_message, None);
            if value_wrapped.is_err() {
                println!("Command setoption did not include any data for value.");
            }

            let value = value_wrapped.unwrap();

            return UCICommands::SetOption { name: option_name, option_type, value: Some(value) }
        }

        if option_type == OptionType::Button {
            return UCICommands::SetOption { name: option_name, option_type: OptionType::Button, value: None }
        }

        UCICommands::Unknown("The function failed".to_string())
    }


    fn parse_position(split_message: Vec<&str>) -> UCICommands {
        let original_message = String::from(split_message.join(" "));

        let fen: String;

        match split_message.get(1) {
            Some(&"fen") =>  {
                let unwrapped_fen = Self::collect_until_end_or_breakpoint(0, &split_message, Some("moves"));

                if unwrapped_fen.is_err() {
                    println!("Command position did not include a fen for the fen parameter.");
                    return UCICommands::Unknown(original_message);
                }

                fen = unwrapped_fen.unwrap();
            }

            Some(&"startpos") =>  { fen = START_POS.to_string() }


            Some(&msg) => {
                println!("Command position does not include the parameter '{msg}'.");
                return UCICommands::Unknown(original_message);
            }

            None => {
                println!("Command position must include a parameter.'");
                return UCICommands::Unknown(original_message);
            },
        }


        let moves_position_wrapped = split_message.iter().position(|&x| x == "value");
        if moves_position_wrapped.is_none() {
            return UCICommands::Position { fen: Some(fen.to_string()), moves: None};
        }

        let moves_position = moves_position_wrapped.unwrap();

        if moves_position+1 == split_message.len() {
            println!("Command position did not include a value for the moves parameter, ignoring moves.");
            return UCICommands::Position { fen: Some(fen.to_string()), moves: None};

        }

        let moves = Some(split_message[moves_position..].iter().map(|x| x.to_string()).collect::<Vec<String>>());
        UCICommands::Position { fen: Some(fen.to_string()), moves}
    }


    fn message_is_u32(message: &str) -> Result<(), ()> {

        let u32_from = u32::from_str(message);

        if u32_from.is_err() {
            println!("Command go parameter value is not a valid positive integer.");
            return Err(());
        }

        Ok(())
    }

    fn parse_go(split_message: Vec<&str>) -> UCICommands {
        let original_message = String::from(split_message.join(" "));

        let mut move_time: Option<u32> = None;
        let mut wtime: Option<u32> = None;
        let mut btime: Option<u32> = None;
        let mut winc: Option<u32> = None;
        let mut binc: Option<u32> = None;
        let mut moves_to_go: Option<u32> = None;


        for i in (0..split_message.len()-1).skip(1).step_by(2) {
            let message_type = split_message[i];
            let message_value = split_message[i+1];

            if Self::message_is_u32(message_value).is_err() { return UCICommands::Unknown(original_message); }
            let as_u32 = Some(u32::from_str(message_value).unwrap());

            match message_type {
                "move_time" => { move_time = as_u32; },
                "wtime" => { wtime = as_u32; },
                "btime" => { btime = as_u32; },
                "winc" => { winc = as_u32; },
                "binc" => { binc = as_u32; },
                "movestogo" => { moves_to_go = as_u32; },

                msg => {
                    println!("Command go does not include '{msg}' as a valid message type.");
                    return UCICommands::Unknown(original_message);
                }


            }
        }

        UCICommands::Go {move_time, wtime, btime, winc, binc, moves_to_go}
    }

    pub fn parse(message: &str) -> UCICommands {

        let split_message: Vec<_> = message.split_whitespace().collect();

        match split_message.get(0){
            None => { return UCICommands::Unknown(String::from(message))  },
            Some(_) => {}
        }

        let initial_command = *split_message.get(0).unwrap();

        match initial_command {
            "uci" => { UCICommands::Uci },
            "isready" => { UCICommands::IsReady },
            "quit" => { UCICommands::Quit },
            "setoption" => { Self::parse_set_option(split_message) },
            "position" => { Self::parse_position(split_message) },
            "go" => { Self::parse_go(split_message) },

            _ => {
                println!("Unknown command: '{}'. Type help for more information.", initial_command);
                UCICommands::Unknown(String::from(initial_command))
            },
        }

    }



}