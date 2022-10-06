use crate::command::Command;
use std::ffi::CStr;
use std::os::unix::process::CommandExt;

pub fn exec(command: &Command) {
    let e = std::process::Command::new(&command.args[1]).args(&command.args[2..]).exec();
    eprintln!("{e}");
}
