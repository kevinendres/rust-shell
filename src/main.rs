use std::io::{self, Write};
use std::ffi::CString;
use std::ffi::CStr;
use nix::unistd::ForkResult;
use std::path::PathBuf;
use std::str::FromStr;

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
        let filename = CString::new(PathBuf::from_str(&input[0]
                                                      .clone()
                                                      .into_string()
                                         .expect("Error CString filename")
                                         .as_str())
                                        .expect("Error pathbuf")
                                        .file_name()
                                    .expect("Error Filename")
                                    .to_str()
                                    .expect("OsStr to Str"))
                                    .expect("CString to filename");
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
                let _execve_result = nix::unistd::execvp(&filename, args);
            },
            ForkResult::Parent {
                child: c } => {
                let _wait_status = nix::sys::wait::waitpid(c, None);
            },
        }
    }
}
