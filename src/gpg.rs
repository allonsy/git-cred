use std::path::Path;
use std::process::Command;
use std::process::Stdio;
use std::io;
use std::io::Write;
use command;

pub fn decrypt<P: AsRef<Path>>(path: P) -> io::Result<String> {
    let output = command::run_command("gpg", vec!["--decrypt", path.as_ref().to_str().unwrap()]);
    if output.is_err() {
        return Err(output.err().unwrap());
    }
    let output_unwrapped = output.unwrap();
    if !output_unwrapped.status.success() {
        return Err(io::Error::new(io::ErrorKind::Other, "Decryption failed"));
    }

    return Ok(String::from_utf8(output_unwrapped.stdout).unwrap());
}

pub fn encrypt<P: AsRef<Path>>(path: P, recipients: &Vec<&str>) -> io::Result<()> {
    let mut args = vec!["--encrypt", "--batch", "--yes"];
    for recipient in recipients {
        args.push("--recipient");
        args.push(recipient);
    }
    args.push(path.as_ref().to_str().unwrap());
    let output = command::run_command("gpg", args);
    if output.is_err() {
        return Err(output.err().unwrap());
    }
    if !output.unwrap().status.success() {
        return Err(io::Error::new(io::ErrorKind::Other, "Encryption failed"));
    }

    return Ok(());
}

pub fn encrypt_string<P: AsRef<Path>>(path: P, contents: String, recipients: &Vec<&str>) -> io::Result<()> {
    let path_str = path.as_ref().to_str().unwrap();
    let mut args = vec!["--encrypt", "--batch", "--yes", "--output", path_str];

    for recipient in recipients {
        args.push("--recipient");
        args.push(recipient);
    }
    
    let mut output = Command::new("gpg")
        .args(args)
        .stdin(Stdio::piped())
        .spawn()
        .expect("Unable to call gpg");
    
    output.stdin.as_mut().unwrap().write_all(&contents.into_bytes()).unwrap();
    let exit_code = output.wait().unwrap();
    if !exit_code.success() {
        return Err(io::Error::new(io::ErrorKind::Other, "Encryption failed"));
    } else {
        return Ok(());
    }
}

pub fn reencrypt<P: AsRef<Path>>(path: P, recipients: &Vec<&str>) -> io::Result<()> {
    let contents = decrypt(&path);
    if contents.is_err() {
        return Err(contents.err().unwrap());
    }

    encrypt_string(path, contents.unwrap(), recipients)
}

pub fn has_key(id: &str) -> bool {
    let output = command::run_command("gpg", vec!["--list-keys", id]).unwrap();
    return output.status.success();
}