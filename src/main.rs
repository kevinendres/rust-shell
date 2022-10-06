mod command;
mod builtins;
mod prompt;
mod execute;

fn main() {
    let mut executor = execute::Executor::new();

    loop {
        prompt::print_prompt(executor.last_status);
        let args = prompt::read_from_stdin();
        let commands = command::parse(&args);
        executor.execute(commands);
    }
}
