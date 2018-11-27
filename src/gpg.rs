use std::path::Path;
use std::io;
use command;

pub fn decrypt<P: AsRef<Path>>(path: P) -> io::Result<String> {
    let output = command::run_command("gpg", vec!["--decrypt", path.as_ref().to_str().unwrap()]);
    return Ok(String::from_utf8(output.unwrap().stdout).unwrap());
}

pub fn has_key(id: &str) -> bool {
    let output = command::run_command("gpg", vec!["--list-keys", id]).unwrap();
    return output.status.success();
}