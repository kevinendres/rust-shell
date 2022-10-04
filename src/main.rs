mod command;
mod builtins;
mod prompt;

fn main() {
    let mut status = 0;
    loop {

        prompt::print_prompt(status);
        let args = prompt::read_from_stdin();
        let commands = command::parse(&args);
        status = commands.execute();

    }
}
