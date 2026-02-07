use std::io;
use std::str::FromStr;

pub fn get_input(prompt: &str) -> String {
    print!("{}", prompt);
    io::Write::flush(&mut io::stdout()).expect("Failed to flush stdout");
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");
    input.trim().to_string()
}

pub fn get_parsed_input<T: FromStr>(prompt: &str) -> T {
    loop {
        let input = get_input(prompt);
        match input.parse() {
            Ok(value) => return value,
            Err(_) => println!("Invalid input. Please try again."),
        }
    }
}

pub fn get_parsed_input_with_default<T: FromStr + Clone>(prompt: &str, default: T) -> T {
    loop {
        let input = get_input(prompt);
        if input.is_empty() {
            return default.clone();
        }
        match input.parse() {
            Ok(value) => return value,
            Err(_) => println!("Invalid input. Please try again."),
        }
    }
}
