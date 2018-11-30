pub fn error_out(message: &str) -> ! {
    eprintln!("{}", message);
    std::process::exit(2)
}
