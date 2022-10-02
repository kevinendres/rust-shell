mod builtins;
mod command;

use std::io::{self, Write};
use std::ffi::CString;
use nix::unistd::ForkResult;
use std::path::{PathBuf, Path};
use std::str::FromStr;

fn main() {
    loop {
        let current_path = std::env::current_dir().expect("Error current_dir");
        let parent_dir = if current_path != PathBuf::from("/") {
            current_path.as_path().parent().expect("Error parent dir")
        }
        else {
            &current_path
        };
        let current_dir = current_path.as_path().strip_prefix(parent_dir).expect("error strip prefix");

        print!("{}: $ ", current_dir.display());
        io::stdout().flush().expect("Error flush");

        let mut buffer = String::new();
        match io::stdin().read_line(&mut buffer) {
            Err(err) => eprintln!("{}", err),
            Ok(0) => break,     //EOF, end shell
            Ok(_) => {}         //bytes read successfully
        }

        let mut input: Vec<&str> = buffer.split_whitespace().collect();
        let filename = match input[0] {
            "cd" => {
                // input[0] = "chdir";
                // input[1] = match input[1] {
                //     ".." => parent_dir.to_str().expect("Error parent_dir to str"),
                //     unchanged => unchanged,
                // };
                // println!("{:?}", input);
                // CString::new("chdir").expect("Error cd to CString")
                builtins::cd::cd(&input);
                continue;
                
            },

            "/bin/pwd" => {
                builtins::pwd::pwd();
                continue;
            }

            bin => CString::new(PathBuf::from_str(bin)
                                        .expect("Error pathbuf")
                                        .file_name()
                                        .expect("Error Filename")
                                        .to_str()
                                        .expect("OsStr to Str"))
                                        .expect("CString to filename"),
        };
        let cstring_input = input
                            .iter()
                            .map(|s| CString::new((s).as_bytes()).expect("Error CString"))
                            .collect::<Vec<CString>>();
        let args = if input.len() > 1 {
                        cstring_input.as_slice()
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
