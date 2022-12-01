use std::os::unix::process::CommandExt;
use std::process;
use anyhow::{anyhow, Result};
use std::env;

pub fn cd(args: &[String]) -> Result<()> {
    let p = std::path::Path::new(&args[1]);
    env::set_current_dir(p)?;

    //MacOs uses symlinks for /var, /tmp, and /etc
    //and redirects them to /private/var, /private/tmp, and /private/etc
    #[cfg(target_os = "macos")]
    if env::current_dir()? == std::path::PathBuf::from("/private") {
        env::set_current_dir("/")?;
    }
    Ok(())
}

/// On success, this function does not return but instead swaps
/// the current process with the called process. Similar to exit,
/// no stack cleanup will be performed or destructors called.
pub fn exec(args: &[String]) -> Result<()> {
    let e = process::Command::new(&args[1])
        .args(&args[2..])
        .exec();
    Err(anyhow!(e))
}

pub fn pwd() -> Result<()> {
    println!("{}", std::env::current_dir()?
             .as_path()
             .display());
    Ok(())
}

pub fn exit(args: &[String]) -> Result<()> {
    let mut code = 0;
    if args.len() > 1 {
        code = args[1].parse::<i32>()?;
    }
    process::exit(code);
}
