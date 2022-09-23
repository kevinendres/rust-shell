use std::io::{self, Write};
use std::ffi::CString;
use std::ffi::CStr;
use nix::unistd::ForkResult;

fn main() {
    loop {
        let current_path = std::env::current_dir().expect("Error current_dir");
        let prefix = current_path.as_path().parent().expect("Error parent dir");
        let _current_dir = current_path.as_path().strip_prefix(prefix).expect("error strip prefix");

        //print!("{}: $ ", current_dir.display());
        print!("$ ");
        io::stdout().flush().expect("Error flush");

        let mut buffer = String::new();
        match io::stdin().read_line(&mut buffer) {
            Err(err) => eprintln!("{}", err),
            Ok(0) => break,     //EOF, end shell
            Ok(_) => {}         //bytes read successfully
        }

        let input: Vec<CString> = buffer
            .split_whitespace()
            .map(|s| CString::new(s).expect("Error CString"))
            .collect();
        let path = &input[0];
        let args = if input.len() > 1 {
                        &input[..]
                    }
                    else {
                        &[] as &[CString]
                    };

        let fork_res;
        unsafe { fork_res = nix::unistd::fork().expect("Error fork"); };

        match fork_res {
            ForkResult::Child => {
                let _execve_result = nix::unistd::execve(path, args, &[] as &[&CStr]);
            },
            ForkResult::Parent {
                child: c } => {
                let _wait_status = nix::sys::wait::waitpid(c, None);
            },
        }
    }
}
