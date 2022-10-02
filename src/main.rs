mod command;

use std::env;
use std::io::Write;
use command::Command;

fn print_prompt(code: i32) {
    let path = env::current_dir().expect("Error! evn::current_dir failed");
    let current_dir = path.file_name().unwrap().to_str();

    match code {
        0 => print!("\x1b[92m{}: ?\x1b[0m ", current_dir.unwrap_or(">>>")),
        _ => print!("\x1b[91m{}: ?\x1b[0m ", current_dir.unwrap_or(">>>")),
    }
    std::io::stdout().flush().expect("Error flush");
}

fn main() {
    let mut last_result = 0;
    loop {

        print_prompt(last_result);
        let command = Command::read_from_stdin();
        last_result = command.execute();

    }
}
