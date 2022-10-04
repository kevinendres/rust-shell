use crate::builtins;

use std::io;
use std::ffi::{CString, CStr};
use nix::unistd::ForkResult;
use std::path::PathBuf;
use std::str::FromStr;
use shell_words::{ParseError};

#[derive(Default, Debug)]
pub struct CommandList(Vec<Command>);

#[derive(Debug)]
pub enum Terminator {
    LogAnd,
    LogOr,
    Pipe,
    SemiCol,
    RedirOut,
    RedirIn,
}

#[derive(Debug)]
pub struct Command {
    args: Vec<String>,
    bang: bool,
    term: Option<Terminator>,
}

pub fn parse(input: &String) -> CommandList {
    let tokens: Vec<String> = match shell_words::split(&input) {
        Err(err)   => { eprintln!("{}", err); Vec::new() },
        Ok(tokens) => { tokens },
    };

    let mut com_list = CommandList::new();
    let mut command = Command::new();
    for token in tokens {
        match token {
            _ if token == "!"  => command.bang = true,
            _ if token == "&&" => {
                command.term = Some(Terminator::LogAnd);
                com_list.add(command);
                command = Command::new();
            },
            _ if token == "||" => {
                command.term = Some(Terminator::LogOr);
                com_list.add(command);
                command = Command::new();
            },
            _ if token == ";"  => {
                command.term = Some(Terminator::SemiCol);
                com_list.add(command);
                command = Command::new();
            },
            _ if token == "|"  => {
                command.term = Some(Terminator::Pipe);
                com_list.add(command);
                command = Command::new();
            },
            _ if token == "<"  => {
                command.term = Some(Terminator::RedirIn);
                com_list.add(command);
                command = Command::new();
            },
            _ if token == ">"  => {
                command.term = Some(Terminator::RedirOut);
                com_list.add(command);
                command = Command::new();
            },
            mut s                  => {
                if s.ends_with(';') {
                    command.term = Some(Terminator::SemiCol);
                    s.pop();
                    command.args.push(s);
                    com_list.add(command);
                    command = Command::new();
                }
                else {
                    command.args.push(s);
                }
            },
        }
    }
    if !command.args.is_empty() {
        com_list.add(command);
    }
    com_list
}

impl Command {
    pub fn new() -> Self {
        Command{ args: Vec::new(), bang: false, term: None }
    }

    pub fn execute(&self) -> i32 {
        match self.args[0].as_str() {
            "cd" => {
                builtins::cd::cd(&self.args);
                return 0;
            },
            "pwd" | "/bin/pwd" => {
                builtins::pwd::pwd();
                return 0;
            },
            _ => {},
        };
        let fork_res;
        unsafe { fork_res = nix::unistd::fork().expect("Error fork"); };

        let result = match fork_res {
            ForkResult::Child => {
                let (bin, args) = self.convert_to_c_string();
                nix::unistd::execvp(bin.as_c_str(), args.as_slice());
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

    fn convert_to_c_string(&self) -> (CString, Vec<CString>) {
        let bin = CString::new(self.args[0].as_bytes()).expect("CString failed bin");
        let mut args = Vec::new();
        for arg in &self.args {
            args.push(CString::new(arg.as_bytes()).expect("CString failed args"));
        }
        (bin, args)
    }
}

impl CommandList {
    pub fn new() -> Self {
        CommandList(Vec::new())
    }

    pub fn execute(&self) -> i32 {
        let mut status = 0;
        for command in &self.0 {
            status = command.execute();
        }
        status
    }

    pub fn add(&mut self, command: Command) {
        self.0.push(command);
    }
}

