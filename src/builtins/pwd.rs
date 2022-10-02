pub fn pwd() {
    println!("{}", std::env::current_dir().expect("pwd failed").as_path().display());
}
