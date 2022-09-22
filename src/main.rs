use std::io::{self, Write};

fn main() {
    loop {
        let current_path = std::env::current_dir().expect("Error current_dir");
        let prefix = current_path.as_path().parent().expect("Error parent dir");
        let current_dir = current_path.as_path().strip_prefix(prefix).expect("error strip prefix");
        print!("{}: $ ", current_dir.display());
        io::stdout().flush().expect("Error flush");
        let mut buffer = String::new();
        io::stdin().read_line(&mut buffer).expect("Error couldn't read stdin");
        if buffer == "exit\n" {
            break;
        }
    }
}
