mod parser;

use nix::unistd::ForkResult;
use nix::sys::wait::WaitStatus;
use conch_parser::lexer::Lexer;
use conch_parser::parse::DefaultParser;
use conch_parser::ast;
use std::process;
use std::collections::VecDeque;
use std::process::{Stdio, ExitStatus};
use std::io::{Write, Read};
use std::os::unix::process::ExitStatusExt;
use std::fs::{File, OpenOptions};
use crate::execute::{Execute, Pipe};
use crate::builtins;
use parser::*;
use anyhow::{anyhow, Result};

pub type TopLevelCommandList = Vec<ast::TopLevelCommand<String>>;

#[derive(Debug)]
pub enum Conjunction {
    And,
    Or,
}

#[derive(Debug)]
pub enum Redirect {
    Read(Option<u16>, String),
    Write(Option<u16>, String),
    Append(Option<u16>, String),
    ReadWrite(Option<u16>, String),
    DupRead(Option<u16>, String),
    DupWrite(Option<u16>, String),
    Clobber(Option<u16>, String),
    Heredoc(Option<u16>, String),
}

pub struct SubshellCommand {
    commands: Vec<Box<dyn Execute>>,
}

pub struct SingleCommand {
    command: Box<dyn Execute>,
}

pub struct SimpleCommand {
    command: process::Command,
}

pub struct PipeCommands {
    pub commands: Vec<Box<dyn Pipe>>,
    bang: bool,
}

pub struct AndOrCommandList {
    first: Box<dyn Execute>,
    rest: Vec<AndOrCommand>,
}

pub struct AndOrCommand {
    command: Box<dyn Execute>,
    conjunction: Conjunction,
}

#[derive(Debug)]
pub struct RedirectCommand {
    command: process::Command,
    redirect: Redirect,
}

#[derive(Debug)]
pub struct BuiltinCommand {
    args: Vec<String>,
}

pub fn parse_into_commands(input: &str) -> Result<Vec<Box<dyn Execute>>> {
    let mut boxed_command_list: Vec<Box<dyn Execute>> = vec![];
    let lex = Lexer::new(input.chars());
    let parser = DefaultParser::new(lex);

    let ast_com_list: TopLevelCommandList = parser.into_iter()
                                                .filter_map(|r| r.map_err(|e| eprintln!("Arsh command parse error: {e}")).ok())
                                                .collect();
    for ast_command in ast_com_list {
        let boxed_command = generate_command(ast_command)?;
        boxed_command_list.push(boxed_command);
    }
    Ok(boxed_command_list)
}

impl Execute for SingleCommand {
    fn execute(&mut self) -> Result<ExitStatus> {
        self.command.execute()
    }

    fn execute_to_string(&mut self) -> Result<String> {
        self.command.execute_to_string()
    }
}

impl Execute for SimpleCommand {
    fn execute(&mut self) -> Result<ExitStatus> {
        let mut child = self.command.spawn()?;
        let status = child.wait()?;
        Ok(status)
    }

    fn execute_to_string(&mut self) -> Result<String> {
        self.command.stdout(Stdio::piped());
        let mut child = self.command.spawn()?;
        child.wait()?;
        let output_opt = child.stdout;
        let mut buf = String::new();
        if let Some(mut output) = output_opt {
            let _bytes_read = output.read_to_string(&mut buf)?;
        }
        Ok(buf)

    }
}

impl Pipe for SimpleCommand {
    fn get_child(&mut self) -> std::process::Child {
        self.command.spawn().unwrap()
    }

    fn pipe_in(&mut self, in_pipe: process::ChildStdout) {
        self.command.stdin(Stdio::from(in_pipe));
    }

    fn pipe_out(&mut self) {
        self.command.stdout(Stdio::piped());
    }
}

fn negate_status(status: ExitStatus) -> ExitStatus {
    if status.success() {
        ExitStatus::from_raw(1)
    }
    else {
        ExitStatus::from_raw(0)
    }
}

impl Execute for PipeCommands {
    fn execute(&mut self) -> Result<ExitStatus> {
        let pipe_status: ExitStatus = if self.commands.len() == 1 {
            self.commands.pop().unwrap().execute()?
        }
        else {
            let mut commands: VecDeque<Box<dyn Pipe>> = self.commands.drain(..).collect();
            //initialize vars before loop
            let mut first = commands.pop_front().unwrap();
            first.pipe_out();
            let mut first_child = first.get_child();
            let mut status = first_child.wait()?;
            let mut output_opt = first_child.stdout;
            while let Some(mut simple_command) = commands.pop_front() {
                if let Some(output) = output_opt {
                    simple_command.pipe_in(output);
                }
                simple_command.pipe_out();
                let mut child = simple_command.get_child();
                status = child.wait()?;
                output_opt = child.stdout;
            }
            if let Some(output) = output_opt {
                let print_out_result: std::result::Result<Vec<u8>, std::io::Error> = output.bytes().collect();
                let print_out = print_out_result.unwrap();
                match std::io::stdout()
                    .lock()
                    .write_all(&print_out) {
                        Ok(_) => {},
                        Err(e) => eprintln!("Failed to write final output of pipe: {}", e),
                    }
            }
            status
        };
        if self.bang {
            Ok(negate_status(pipe_status))
        }
        else {
            Ok(pipe_status)
        }
    }

    fn execute_to_string(&mut self) -> Result<String> {
        if self.commands.len() == 1 {
            self.commands.pop().unwrap().execute_to_string()
        }
        else {
            let mut commands: VecDeque<Box<dyn Pipe>> = self.commands.drain(..).collect();
            //initialize vars before loop
            let mut first = commands.pop_front().unwrap();
            first.pipe_out();
            let mut first_child = first.get_child();
            let mut _status = first_child.wait()?;
            let mut output_opt = first_child.stdout;
            while let Some(mut simple_command) = commands.pop_front() {
                if let Some(output) = output_opt {
                    simple_command.pipe_in(output);
                }
                simple_command.pipe_out();
                let mut child = simple_command.get_child();
                _status = child.wait()?;
                output_opt = child.stdout;
            }
            let mut buf = String::new();
            if let Some(mut output) = output_opt {
                let _bytes_read = output.read_to_string(&mut buf);
            }
            Ok(buf)
        }
    }
}

impl Execute for AndOrCommandList {
    fn execute(&mut self) -> Result<ExitStatus> {
        let mut status = match self.first.execute() {
            Ok(status) => status,
            Err(e)     => {
                eprintln!{"Execution error: {e}"};
                ExitStatus::from_raw(1)
            }
        };
        for command in &mut self.rest {
            status = match command.conjunction {
                Conjunction::And => {
                    if status.success() {
                        command.execute()?
                    }
                    else {
                        ExitStatus::from_raw(1)
                    }
                },
                Conjunction::Or => {
                    if !status.success() {
                        command.execute()?
                    }
                    else {
                        ExitStatus::from_raw(0)
                    }
                },

            }
        }
        Ok(status)
    }

    fn execute_to_string(&mut self) -> Result<String> {
        let mut output = match self.first.execute_to_string() {
            Ok(output) => { output },
            Err(e)     => {
                eprintln!{"Execution Error: {e}"};
                String::new()
            }
        };
        let mut res_string;
        for command in &mut self.rest {
            output += match command.conjunction {
                Conjunction::And => {
                    if !output.is_empty() {
                        res_string = command.execute_to_string()?;
                        res_string.trim()
                    }
                    else {
                        ""
                    }
                },
                Conjunction::Or => {
                    if output.is_empty() {
                        res_string = command.execute_to_string()?;
                        res_string.trim()
                    }
                    else {
                        ""
                    }
                },

            }
        }
        Ok(output)
    }
}

impl Execute for AndOrCommand {
    fn execute(&mut self) -> Result<ExitStatus> {
        self.command.execute()
    }

    fn execute_to_string(&mut self) -> Result<String> {
        self.command.execute_to_string()
    }
}

impl Execute for RedirectCommand {
    fn execute(&mut self) -> Result<ExitStatus> {
        self.add_redirect_to_command()?;
        let mut child = self.command.spawn()?;
        let status = child.wait()?;
        Ok(status)

    }

    fn execute_to_string(&mut self) -> Result<String> {
        self.command.stdout(Stdio::piped());
        self.add_redirect_to_command()?;
        let mut child = self.command.spawn()?;
        child.wait()?;
        let output_opt = child.stdout;
        let mut buf = String::new();
        if let Some(mut output) = output_opt {
            let _bytes_read = output.read_to_string(&mut buf)?;
        }
        Ok(buf)
    }
}

impl Pipe for RedirectCommand {
    fn get_child(&mut self) -> std::process::Child {
        match self.add_redirect_to_command() {
            Ok(_)  => {},
            Err(e) => eprintln!("Failed to add redirect in pipe: {}", e),
        }
        self.command.spawn().unwrap()
    }

    fn pipe_in(&mut self, in_pipe: process::ChildStdout) {
        self.command.stdin(Stdio::from(in_pipe));
    }

    fn pipe_out(&mut self) {
        self.command.stdout(Stdio::piped());
    }
}

impl RedirectCommand {
    fn add_redirect_to_command(&mut self) -> Result<()> {
        match &self.redirect {
            Redirect::Read(fd, filename) => {
                let file = File::open(filename)?;
                match fd {
                    None    => self.command.stdin(Stdio::from(file)),
                    Some(0) => self.command.stdin(Stdio::from(file)),
                    _       => { return Err(anyhow!("Implement Redirect::Read from arbitrary FDs")); }
                };
            }
            Redirect::Write(fd, filename) => {
                let file = OpenOptions::new()
                    .create(true)
                    .truncate(true)
                    .write(true)
                    .open(filename)?;
                match fd {
                    None    => self.command.stdout(Stdio::from(file)),
                    Some(1) => self.command.stdout(Stdio::from(file)),
                    Some(2) => self.command.stderr(Stdio::from(file)),
                    _       => { return Err(anyhow!("Implement Redirect::Write from arbitrary FDs")); }
                };
            }
            Redirect::Append(fd, filename) => {
                let file = OpenOptions::new()
                    .create(true)
                    .append(true)
                    .write(true)
                    .open(filename)?;
                match fd {
                    None    => self.command.stdout(Stdio::from(file)),
                    Some(1) => self.command.stdout(Stdio::from(file)),
                    Some(2) => self.command.stderr(Stdio::from(file)),
                    _       => { return Err(anyhow!("Implement Redirect::Append from arbitrary FDs")); }
                };
            }
            Redirect::ReadWrite(fd, filename) => {
                let file = OpenOptions::new()
                    .create(true)
                    .write(true)
                    .read(true)
                    .open(filename)?;
                match fd {
                    None    => self.command.stdin(Stdio::from(file)),
                    Some(0) => self.command.stdin(Stdio::from(file)),
                    Some(1) => self.command.stdout(Stdio::from(file)),
                    Some(2) => self.command.stderr(Stdio::from(file)),
                    _       => { return Err(anyhow!("Implement Redirect::ReadWrite from arbitrary FDs")); }
                };
            }
            Redirect::Clobber(_, _com) => { return Err(anyhow!("Implement Redirect::Clobber")); }
            Redirect::Heredoc(_, _com) => { return Err(anyhow!("Implement Redirect::Heredoc")); }
            Redirect::DupRead(_, _com) => { return Err(anyhow!("Implement Redirect::DupRead")); }
            Redirect::DupWrite(_n, _m) => { return Err(anyhow!("Implement Redirect::DupWrite")); }
        };
        Ok(())
    }
}

impl Execute for BuiltinCommand {
    fn execute(&mut self) -> Result<ExitStatus> {
        if let Some(builtin) = self.args.first() {
            match builtin.as_str() {
                "cd" => {
                    builtins::cd(&self.args)?;
                },
                "pwd" | "/bin/pwd" => {
                    builtins::pwd()?;
                },
                "exec" => {
                    builtins::exec(&self.args)?;
                }
                "exit" => {
                    builtins::exit(&self.args)?;
                }
                _ => {
                    return Err(anyhow!("Malformed builtin"));
                },
            }
        }
        else {
            return Err(anyhow!("Malformed builtin"));
        }
        Ok(ExitStatus::from_raw(0))
    }

    fn execute_to_string(&mut self) -> Result<String> {
        Err(anyhow!("String versions of builtins not yet implemented"))
    }
}

impl Execute for SubshellCommand {
    fn execute(&mut self) -> Result<ExitStatus> {
        match unsafe{ nix::unistd::fork()? } {
            ForkResult::Parent{child: _} => {
                match nix::sys::wait::wait()? {
                    WaitStatus::Exited(_pid, code) => Ok(ExitStatus::from_raw(code)),
                    _ => {
                        eprintln!("waitpid exit error");
                        Ok(ExitStatus::from_raw(1))
                    }
                }
            },
            ForkResult::Child => {
                let mut status = ExitStatus::from_raw(0);
                for command in &mut self.commands {
                    status = command.execute()?;
                }
                let code = if let Some(code) = status.code() {
                    code
                }
                else { 0 };
                std::process::exit(code)
            },
        }
    }

    fn execute_to_string(&mut self) -> Result<String> {
        Err(anyhow!("Cannot print subshell to string"))
    }
}
