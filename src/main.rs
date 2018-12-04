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
        "help" => {
            print_help(command_args);
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

fn print_help(args: &[String]) {
    if args.is_empty() {
        let help_str = "git-cred: an encrypted git credential helper
        commands:
            init: init a new credential store or subfolder
            encrypt: encrypt a file or string in the store
            decrypt: decrypt a file in the store
            reencrypt: reencrypt a folder (or the whole store) with new gpg ids
            save-key: save a key in the repo for other users to use
        
        To see more detailed instructions for each subcommand
            run: git cred help [subcommand]
        ";
        println!("{}", help_str);
        return;
    }

    let subcommand = &args[0];

    let help_str = match subcommand.as_ref() {
        "init" => {
            "git cred init help
Init a credential store or subfolder within the store

usage: git cred init [-f folder_name] [gpg_ids...]
    -f folder_name: instead of intializing the root folder of the credential store,
                    intialize a subfolder.
    
    [gpg_ids...]:   a space separated list of gpg ids to use for encryption.
                    These gpgs will be specific to that folder
                    (or root folder if none specified). These can be emails, gpg key ids, or github usernames
                    if no gpgs are provided, the field of user.email in your git config is used.

Notes:
    If the folder you provide (or the root folder) already exists, the existing gpg ids
    will be overwritten and the target folder will be automatically reencrypted with the provided gpg_ids

Examples:
    git cred init
    git cred init -f foo/bar
    git cred init username1
    git cred init username1 email@email.com
    git cred init -f foo username1 email@email.com
"
        }
        "encrypt" => {
            "git cred encrypt help
Encrypt a file or string in the store

usage: git cred encrypt path_to_encrypt (-f file_name | string_to_encrypt)
    path_to_encrypt:    the location in the credential store you want to encrypt to
                        e.g. /foo/bar
    -f file_name:       you may provide a file that will be encrypted via this flag
    string_to_encrypt:  instead of providing a file, you can simply write the string to encrypt
                        as a single command line argument

Examples:
    git cred encrypt foo hello
    git cred encrypt foo \"hello, world!\"
    git cred encrypt foo/bar hello
    git cred encrypt foo -f secret.txt"
        }
        "decrypt" => {
            "git cred decrypt help
Decrypt a file in the store

usage: git cred decrypt file_path
    file_path:  the file path to decrypt. The decrypted string will be output to standard out

Examples:
    git cred decrypt foo
    git cred decypt foo/bar"
        }
        "reencrypt" => {
            "git cred reencrypt help
Reencrypt your credential store based on the gpg ids already present in the store.
To reencrypt with different gpg ids, use 'git cred init' instead.

usage:  git cred reencrypt

Examples:
    git cred reencrypt"
        }
        "save-key" => {
"git cred save-key
Store a public key from your gpg keyring/github into the repo

usage: git cred save-key (uid) [keyfile]
    uid:        Either a github username, email, or gpg key id
                The key for that uid will be looked up and the public key will
                be inserted into the repo for other users to encrypt with
    [keyfile]:  instead of looking up the public key for that uid, the public key in
                the provided keyfile will be saved instead. All further encryption
                for that uid will use that key instead. You may remove the keyfile
                after this command completes

Notes:
    This function can be useful to set the desired key if a given email has more than
    one public key associated with it.

Examples:
    git cred save-key username1
    git cred save-key AAABBBCCC
    git cred save-key email@email.com
    git cred save-key email@email.com /path/to/keyfile.asc
"
        }
        _ => {
            error_out(&format!("Unknown subcommand: {}", subcommand));
        }
    };
    println!("{}", help_str);

}