extern crate git2;
extern crate reqwest;
extern crate serde_json;

mod git_utils;
mod git_config;
mod command;
mod gpg;
mod encrypt;
mod decrypt;
mod resolver;
mod util;
mod github;

use git2::Repository;
use std::path::Path;
use util::error_out;


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
            handle_init(&repo, command_args);
        }
        "encrypt" => {
            handle_encrypt(&repo, command_args);
        }
        "reencrypt" => {
            handle_reencrypt(&repo, command_args);
        }
        "decrypt" => {
            let path = Path::new(&command_args[0]);
            println!("{}", decrypt::decrypt(&repo, path));
        }
        "save-key" => {
            handle_set_user_key(&repo, command_args);
        }
        _ => {
            error_out(&format!("Command not recognized: {}", command));
        }
    };
}

fn handle_init(repo: &Repository, args: &[String]) {
    if args.len() == 0 {
        git_utils::get_credentials_dir(&repo);
        return;
    }

    if args[0] == "-f" {
        if args.len() <= 1 {
            error_out("Please provide folder to init '-f' flag");
        }
        let subfolder = &args[1];
        git_utils::create_sub_dir(repo, &Path::new(subfolder), args[2..].to_vec());
        let path_to_subfolder = git_utils::get_credentials_dir(repo).join(subfolder);
        encrypt::reencrypt_folder(repo, &path_to_subfolder);
        return;
    }

    git_utils::create_sub_dir(repo, Path::new(""), args[0..].to_vec());
    encrypt::reencrypt_folder(repo, &git_utils::get_credentials_dir(repo));
}

fn handle_reencrypt(repo: &Repository, _: &[String]) {
    encrypt::reencrypt_folder(repo, &git_utils::get_credentials_dir(repo));
}

fn handle_encrypt(repo: &Repository, args: &[String]) {
    if args.len() == 0 {
        error_out("Please provide path to the desired encrypted file");
    }

    let path = Path::new(&args[0]);

    if args.len() <= 1 {
        error_out("Please provide either a string or file to encrypt");
    }

    if args[1] == "-f".to_string() {
        if args.len() <= 2 {
            error_out("'-f' flag requires a file name");
        }
        let file_name = &args[2];
        encrypt::encrypt_file(&repo, path, &file_name);
    } else {
        encrypt::encrypt_string(&repo, path, args[1].clone());
    }
}

fn handle_set_user_key(repo: &Repository, args: &[String]) {
    if args.len() == 0 {
        error_out("Please provide a username/email/keyid to save-key");
    }

    let uid = &args[0];

    if args.len() == 1 {
        let key_id = resolver::resolve_name(repo, uid);
        let pub_key = gpg::export_key(&key_id);
        resolver::set_key(repo, uid, &pub_key.unwrap());
        return;
    }

    let filename = &args[1];
    let file_contents = std::fs::read_to_string(filename);
    if file_contents.is_err() {
        error_out(&format!("Unable to read public key from file: {}", filename));
    }

    let pub_key = file_contents.unwrap();

    resolver::set_key(repo, uid, &pub_key);
}