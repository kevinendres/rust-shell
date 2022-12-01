pub mod command;
pub mod builtins;
pub mod prompt;
pub mod execute;
use std::process::ExitStatus;
use std::os::unix::process::ExitStatusExt;

fn main() {
    let mut status = ExitStatus::from_raw(0);

    loop {
        prompt::print_prompt(&status);
        let input = prompt::read_from_stdin();

        let commands = command::parse_into_commands(&input);
        match commands {
            Err(e)       => eprintln!("error parsing commands: {}", e),
            Ok(commands) => status = execute::execute(commands),
        }
    }
}
