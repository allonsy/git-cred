use git_utils;
use gpg;
use git2::Repository;
use std::path::Path;
use std::fs::File;
use std::fs;
use std::io::prelude::Read;
use resolver;

pub fn encrypt_file(repo: &Repository, path: &Path, fname: &str) {
    let mut contents = String::new();

    let file_res = File::open(fname);
    if file_res.is_err() {
        panic!("Unable to open file: {}", fname);
    }
    let mut file = file_res.unwrap();

    file.read_to_string(&mut contents).unwrap();
    encrypt_string(repo, path, contents);
}

pub fn encrypt_string(repo: &Repository, path: &Path, contents: String) {
    let gpgs = get_gpgs_for_file(repo, path);
    let path_to_encrypted_file = git_utils::get_credentials_dir(repo).join(path);

    let mut gpg_pointers: Vec<&str> = Vec::new();
    for gpg in &gpgs {
        gpg_pointers.push(gpg);
    }
    
    gpg::encrypt_string(&path_to_encrypted_file, contents, &gpg_pointers).unwrap();
}

pub fn reencrypt_file(repo: &Repository, path: &Path) {
    let gpgs = get_gpgs_for_file(repo, path);
    let path_to_encrypted_file = git_utils::get_credentials_dir(repo).join(path);

    let mut gpg_pointers: Vec<&str> = Vec::new();
    for gpg in &gpgs {
        gpg_pointers.push(gpg);
    }

    gpg::reencrypt(&path_to_encrypted_file, &gpg_pointers).unwrap();
}

pub fn reencrypt_folder(repo: &Repository, path: &Path) {
    for file in fs::read_dir(path).unwrap() {
        let file_res = file.unwrap();
        let file_name = file_res.file_name().into_string().unwrap();
        if !file_name.starts_with(".") {
            if file_res.file_type().unwrap().is_dir() {
                reencrypt_folder(repo, &file_res.path());
            } else {
                let total_path = file_res.path();
                let relative_path = total_path.strip_prefix(git_utils::get_credentials_dir(repo)).unwrap();
                reencrypt_file(repo, &relative_path);
            }
        }
    }
}

/* Creates directories as needed */
fn get_gpgs_for_file(repo: &Repository, sub_path: &Path) -> Vec<String> {
    let cred_path = git_utils::get_credentials_dir(repo);
    let gpgs = read_gpg_id_file(&cred_path);
    if gpgs.is_none() {
        panic!("No .gpg_id file in credential store");
    }
    let sub_paths = sub_path.to_str().unwrap().split(std::path::MAIN_SEPARATOR).collect();
    let gpgs = get_gpgs_for_file_recursive(&cred_path, sub_paths, gpgs.unwrap());
    
    let mut resolved_gpgs = Vec::new();
    for gpg in &gpgs {
        resolved_gpgs.push(resolver::resolve_name(repo, gpg));
    }
    return resolved_gpgs;
}

fn get_gpgs_for_file_recursive(path: &Path, sub_path: Vec<&str>, gpgs: Vec<String>) -> Vec<String> {
    let mut sub_path = sub_path;
    let mut gpgs = gpgs;

    if sub_path.len() <= 1 {
        return gpgs;
    }
    let this_path = path.to_path_buf().join(sub_path.remove(0));
    if !this_path.exists() {
        fs::create_dir(&this_path).unwrap();
        return get_gpgs_for_file_recursive(&this_path, sub_path, gpgs);
    }
    let this_gpg = read_gpg_id_file(&this_path);
    if this_gpg.is_some() {
        gpgs = this_gpg.unwrap();
    }

    get_gpgs_for_file_recursive(&this_path, sub_path, gpgs)
}

fn read_gpg_id_file(p: &Path) -> Option<Vec<String>> {
    let mut path = p.to_path_buf();
    path = path.join(".gpg_id");

    let file_res = File::open(path);
    if file_res.is_err() {
        return None;
    }

    let mut file = file_res.unwrap();

    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    let mut gpg_ids = Vec::new();
    for gpg in contents.lines() {
        gpg_ids.push(gpg.to_string());
    }
    return Some(gpg_ids);
}