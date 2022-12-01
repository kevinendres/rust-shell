use crate::command::{TopLevelCommandList};
use std::process::{self, ExitStatus};
use std::os::unix::process::ExitStatusExt;
use anyhow::{Result};
use std::fmt;

pub trait Execute {
    fn execute(&mut self) -> Result<ExitStatus>;
    fn execute_to_string(&mut self) -> Result<String>;
}

pub trait Pipe: Execute {
    fn get_child(&mut self) -> std::process::Child;
    fn pipe_in(&mut self, in_pipe: process::ChildStdout);
    fn pipe_out(&mut self);
}

#[derive(Debug, Clone)]
pub struct UnrecognizedCommandError {
    pub message: String
}

impl std::error::Error for UnrecognizedCommandError { }

impl fmt::Display for UnrecognizedCommandError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}, at ({}:{})", self.message, line!(), column!())
    }
}

#[derive(Debug)]
pub struct Executor {
    pub history: Vec<TopLevelCommandList>,
    pub last_status: ExitStatus,
}

pub fn execute(mut commands: Vec<Box<dyn Execute>>) -> ExitStatus {
    let mut status = ExitStatus::from_raw(0);
    for command in &mut commands {
        status = match command.execute() {
            Ok(status) => status,
            Err(msg)   => {
                eprintln!("Execution error: {}", msg);
                ExitStatus::from_raw(1)
            }
        }
    }
    status
}
