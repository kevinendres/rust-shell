use crate::builtins;
use crate::prompt;

use std::io;
use std::ffi::{CString, CStr};
use nix::unistd::ForkResult;
use std::path::PathBuf;
use std::str::FromStr;
use shell_words::{ParseError};

#[derive(Default, Debug)]
pub struct CommandList(pub Vec<Command>);

#[derive(Debug, PartialEq, Eq)]
pub enum Conjunction {
    And,
    Or,
    Pipe,
    SemiCol,
    RedirOut,
    RedirIn,
}

#[derive(Debug)]
pub struct Command {
    pub args: Vec<String>,
    bang: bool,
    pub conj: Option<Conjunction>,
}

pub fn parse(input: &str) -> CommandList {
    let tokens: Vec<String> = match shlex::split(input) {
        None   => { eprintln!("No input parsed"); Vec::new() },
        Some(tokens) => tokens,
    };

    let mut com_list = CommandList::new();
    let mut command = Command::new();

    parse_tokens(tokens, &mut command, &mut com_list);

    if !command.is_empty() {
        com_list.add(command);
    }
    else {
        cont_parse_new_line(&mut command, &mut com_list);
    }
    com_list
}

fn parse_tokens(tokens: Vec<String>, command: &mut Command, com_list: &mut CommandList) {
    for token in tokens {
        match token {
            _ if token == "!"  => command.bang = true,
            _ if token == "&&" => {
                com_list.add(std::mem::replace(command, Command::new()));
                command.conj = Some(Conjunction::And);
            },
            _ if token == "||" => {
                com_list.add(std::mem::replace(command, Command::new()));
                command.conj = Some(Conjunction::Or);
            },
            _ if token == ";"  => {
                com_list.add(std::mem::replace(command, Command::new()));
                command.conj = Some(Conjunction::SemiCol);
            },
            _ if token == "|"  => {
                com_list.add(std::mem::replace(command, Command::new()));
                command.conj = Some(Conjunction::Pipe);
            },
            _ if token == "<"  => {
                com_list.add(std::mem::replace(command, Command::new()));
                command.conj = Some(Conjunction::RedirIn);
            },
            _ if token == ">"  => {
                com_list.add(std::mem::replace(command, Command::new()));
                command.conj = Some(Conjunction::RedirOut);
            },
            // a lone backslash parses as an empty slice
            _ if token.is_empty() => { cont_parse_new_line(command, com_list); },
            mut s                  => {
                if s.ends_with(';') {
                    s.pop();
                    command.args.push(s);
                    com_list.add(std::mem::replace(command, Command::new()));
                    command.conj = Some(Conjunction::SemiCol);
                }
                else {
                    command.args.push(s);
                }
            },
        }
    }
}

fn cont_parse_new_line(command: &mut Command, com_list: &mut CommandList) {
    prompt::print_cont_prompt();
    let new_input = prompt::read_from_stdin();
    if command.is_empty() {
        let new_com_list = parse(&new_input);
        com_list.append(new_com_list);
    }
    else {
        let new_tokens: Vec<String> = match shlex::split(&new_input) {
            None   => { eprintln!("Error no input to lex"); Vec::new() },
            Some(tokens) => tokens,
        };
        parse_tokens(new_tokens, command, com_list);
    };
}

impl Command {
    pub fn new() -> Self {
        Command{ args: Vec::new(), bang: false, conj: None }
    }

    pub fn is_empty(&self) -> bool {
        self.args.is_empty()
    }

    pub fn execute(&self) -> i32 {
        match self.args[0].as_str() {
            "cd" => {
                builtins::cd::cd(&self.args);
                return self.exit(0);
            },
            "pwd" | "/bin/pwd" => {
                builtins::pwd::pwd();
                return self.exit(0);
            },
            "exec" => {
                builtins::exec::exec(self);
                println!("exec fail");
                return self.exit(0);
            }
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
                    Some(nix::sys::wait::WaitStatus::Exited(pid, code))   => self.exit(code),
                                                                        _ => self.exit(1),
                }
            },
        };
        result
    }

    pub fn convert_to_c_string(&self) -> (CString, Vec<CString>) {
        let bin = CString::new(self.args[0].as_bytes()).expect("CString failed bin");
        let mut args = Vec::new();
        for arg in &self.args {
            args.push(CString::new(arg.as_bytes()).expect("CString failed args"));
        }
        (bin, args)
    }

    pub fn exit(&self, code: i32) -> i32 {
        if self.bang {
            match code {
                0 => 1,
                _ => 0,
            }
        }
        else {
            code
        }
    }
}

impl CommandList {
    pub fn new() -> Self {
        CommandList(Vec::new())
    }

    // pub fn execute(&self) -> i32 {
    //     let mut status = 0;
    //     for command in &self.0 {
    //         status = command.execute();
    //     }
    //     status
    // }

    pub fn add(&mut self, command: Command) {
        self.0.push(command);
    }

    pub fn append(&mut self, com_list: CommandList) {
        self.0.extend(com_list.0.into_iter());
    }
}
