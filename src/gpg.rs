use std::path::Path;
use std::process::Command;
use std::process::Stdio;
use std::io;
use std::io::Write;
use command;
use util;

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

pub fn encrypt_string<P: AsRef<Path>>(path: P, contents: String, recipients: &Vec<&str>) -> io::Result<()> {
    let path_str = path.as_ref().to_str().unwrap();
    let mut args = vec!["--encrypt", "--trust-model", "always", "--batch", "--yes", "--output", path_str];

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

pub fn get_key_id(id: &str) -> String {
    let output = command::run_command("gpg", vec!["--keyid-format", "LONG", "-k", id]).unwrap();
    let output_string: String = String::from_utf8(output.stdout).unwrap();

    output_string.lines().nth(1).unwrap().trim().to_string()
}

pub fn import_key(key_contents: &str) -> String {
    return import_key_bytes(&key_contents.to_string().into_bytes());
}

// returns the key-id of the imported key
pub fn import_key_bytes(key_contents: &Vec<u8>) -> String {
    let mut output = Command::new("gpg")
        .args(vec!["--import"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Unable to call gpg");
    
    output.stdin.as_mut().unwrap().write_all(key_contents).unwrap();
    

    let exit_cond = output.wait_with_output().unwrap();
    if !exit_cond.status.success() {
        let output_str = String::from_utf8(exit_cond.stderr).unwrap();
        println!("error is: {}", output_str);
        util::error_out("Unable to import key!");
    }

    let output_str = String::from_utf8(exit_cond.stderr).unwrap();

    let output_lines: Vec<&str> = output_str.lines().collect();
    let first_line = output_lines[0];
    let words: Vec<&str> = first_line.split(" ").collect();
    let mut short_id = words[2].to_string();
    let short_id_len = short_id.len();
    short_id.remove(short_id_len - 1);
    return get_key_id(&short_id);
}

pub fn export_key(uid: &str) -> Option<String> {
    let output = command::run_command("gpg", vec!["--export",  "--armor", uid]);
    if output.is_err() {
        return None;
    }

    return Some(String::from_utf8(output.unwrap().stdout).unwrap());
}