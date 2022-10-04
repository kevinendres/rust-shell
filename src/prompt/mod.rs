use std::env;
use std::io::{self, Write};

pub fn print_prompt(code: i32) {
    let path = env::current_dir().expect("Error! evn::current_dir failed");
    let current_dir = path.file_name().unwrap_or_else(|| std::ffi::OsStr::new("/")).to_str();

    match code {
        0 => print!("\x1b[92m{}: ?\x1b[0m ", current_dir.unwrap_or(">>>")),
        _ => print!("\x1b[91m{}: ?\x1b[0m ", current_dir.unwrap_or(">>>")),
    }
    std::io::stdout().flush().expect("Error flush");
}

pub fn read_from_stdin() -> String {
    let mut input = String::new();
    match io::stdin().read_line(&mut input) {
        Err(err) => eprintln!("{}", err),
        Ok(0) => std::process::exit(0),     //EOF, end shell
        Ok(_) => {}         //bytes read successfully
    }
    input
}
