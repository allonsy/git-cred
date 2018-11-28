extern crate git2;
extern crate reqwest;

mod git_utils;
mod git_config;
mod command;
mod gpg;
mod encrypt;
mod decrypt;

use git2::Repository;
use std::path::Path;


fn main() {
    let repo_res = Repository::discover(".");

    if repo_res.is_err() {
        panic!("Current directory isn't a git repo!");
    }

    let repo = repo_res.unwrap();

    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        error_out("Please provide an command to perform on the credential store");
    }

    let command = &args[1];
    let command_args = &args[2..];

    match command.as_str() {
        "init" => {
            git_utils::get_credentials_dir(&repo);
        }
        "encrypt" => {
            let path = Path::new(&command_args[0]);
            encrypt::encrypt_string(&repo, &path, command_args[1].to_string());
        }
        "decrypt" => {
            let path = Path::new(&command_args[0]);
            println!("{}", decrypt::decrypt(&repo, path));
        }
        _ => {
            error_out(&format!("Command not recognized: {}", command));
        }
    };
}

fn error_out(message: &str) -> ! {
    eprintln!("{}", message);
    std::process::exit(2)
}
