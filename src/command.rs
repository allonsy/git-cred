use std::process::Command;
use std::io::Result;
use std::process::Output;


pub fn run_command(command: &str, args: Vec<&str>) -> Result<Output> {
    let mut com = Command::new(command);
    for arg in args {
        com.arg(arg);
    }
    com.output()
}