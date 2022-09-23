use std::io::{self, Write};

fn main() {
    loop {
        let current_path = std::env::current_dir().expect("Error current_dir");
        let prefix = current_path.as_path().parent().expect("Error parent dir");
        let current_dir = current_path.as_path().strip_prefix(prefix).expect("error strip prefix");

        print!("{}: $ ", current_dir.display());
        io::stdout().flush().expect("Error flush");

        let mut buffer = String::new();
        match io::stdin().read_line(&mut buffer) {
            Err(err) => eprintln!("{}", err),
            Ok(0) => break,     //EOF, end shell
            Ok(_) => {}         //bytes read successfully
        }

        let input: Vec<&str> = buffer.split(' ').collect();

        let mut command = std::process::Command::new(input[0]);
        command.args(&input[1..]);

        command.output().expect("failed to execute command");
    }
}
