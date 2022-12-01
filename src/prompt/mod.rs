use std::env;
use std::path::PathBuf;
use std::process::ExitStatus;
use std::io::{self, Write};

pub fn print_prompt(exit_status: &ExitStatus) {
    let path = match env::current_dir() {
        Ok(path) => path,
        Err(e)   => {
            eprintln!("Error! env::current_dir failed: {}", e);
            PathBuf::from("/")
        }

    };
    let current_dir = path
        .file_name()
        .unwrap_or_else(|| std::ffi::OsStr::new("/"))
        .to_str();

    if exit_status.success() {
        print!("\x1b[92;1m{}: ?\x1b[0m ", current_dir.unwrap_or(">>>"));
    }
    else {
        print!("\x1b[91;1m{}: ?\x1b[0m ", current_dir.unwrap_or(">>>"));
    }
    match std::io::stdout().flush() {
        Ok(_)  => {},
        Err(e) => eprintln!("Couldn't flush stdout: {}", e),
    }
}

pub fn print_cont_prompt() {
    print!("> ");
    match std::io::stdout().flush() {
        Ok(_)  => {},
        Err(e) => eprintln!("Couldn't flush stdout: {}", e),
    }
}

pub fn read_from_stdin() -> String {
    let mut input = String::new();
    loop {
        match io::stdin().read_line(&mut input) {
            Err(err) => eprintln!("Prompt error: {}", err),
            Ok(0) => std::process::exit(0),     //EOF, end shell
            Ok(_) => {},                        //bytes read successfully
        }
        if input.ends_with("&&\n") || input.ends_with("||\n")
            || input.ends_with("|\n") || input.ends_with(">\n")
            || input.ends_with(">>\n") || input.ends_with("<\n") {
            input.pop();   //remove newline char to concat input across lines
            print_cont_prompt();
        }
        else if input.ends_with("\\\n") {
            print_cont_prompt();
            input.pop();   //remove newline char to concat input across lines
            input.pop();   //don't escape first char of next line
        }
        else {
            break;
        }
    }
    input
}
