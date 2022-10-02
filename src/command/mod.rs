use std::io;
use std::ffi::CString;
use nix::unistd::ForkResult;
use std::path::PathBuf;
use std::str::FromStr;

pub struct Command {
    bin: CString,
    args: Vec<CString>,
}

impl Command {
    pub fn read_from_stdin<'a>() -> Command {
        let mut buffer = String::new();
        match io::stdin().read_line(&mut buffer) {
            Err(err) => eprintln!("{}", err),
            Ok(0) => std::process::exit(0),     //EOF, end shell
            Ok(_) => {}         //bytes read successfully
        }

        let mut input: Vec<&str> = buffer.split_whitespace().collect();
        let bin = match input[0] {
            "cd" => {
                CString::new("cd").expect("CString failure")
            },
            "pwd" => {
                CString::new("pwd").expect("CString failure")

            }
            "/bin/pwd" => {
                CString::new("pwd").expect("CString failure")
            }

            bin => CString::new(PathBuf::from_str(bin)
                                        .expect("Error pathbuf")
                                        .file_name()
                                        .expect("Error Filename")
                                        .to_str()
                                        .expect("OsStr to Str"))
                                        .expect("CString to filename"),
        };
        let args = input
                        .iter()
                        .map(|s| CString::new((s).as_bytes()).expect("Error CString"))
                        .collect::<Vec<CString>>();

        Command { bin, args }
    }

    pub fn execute(&self) -> i32 {
        let fork_res;
        unsafe { fork_res = nix::unistd::fork().expect("Error fork"); };

        let result = match fork_res {
            ForkResult::Child => {
                nix::unistd::execvp(&self.bin, self.args.as_slice()).expect("Exec fail");
                1
            },
            ForkResult::Parent {
                child: c } => {
                match nix::sys::wait::waitpid(c, None).ok() {
                    Some(nix::sys::wait::WaitStatus::Exited(pid, code)) => code,
                                                                      _ => 1,
                }
            },
        };
        result
    }
}
