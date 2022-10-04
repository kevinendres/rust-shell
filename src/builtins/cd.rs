use std::env;

pub fn cd(args: &Vec<String>) {
    let p = std::path::Path::new(&args[1]);
    env::set_current_dir(p).expect("CD failed");

    //MacOs uses symlinks for /var, /tmp, and /etc
    //and redirects them to /private/var, /private/tmp, and /private/etc
    #[cfg(target_os = "macos")]
    if env::current_dir().expect("current_dir failed") == std::path::PathBuf::from("/private") {
        env::set_current_dir("/").expect("Macos CD failed");
    }

}
