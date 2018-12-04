use git_utils;
use gpg;
use git2::Repository;
use std::path::PathBuf;
use std::fs;
use util::error_out;
use github;

const KEY_FOLDER_NAME: &'static str = ".keys";
const LOCK_FOLDER_NAME: &'static str = "locks";
const PUBLIC_KEY_FOLDER_NAME: &'static str = "keys";

pub fn resolve_name(repo: &Repository, uid: &str) -> String {
    let key_id = get_locked_key_id(repo, uid);
    if key_id.is_none() {
        if gpg::has_key(uid) {
            let new_key_id = gpg::get_key_id(uid);
            write_lock_file(repo, uid, &new_key_id);
            return new_key_id;
        } else {
            let ghub_key = github::get_key(uid, None);
            if ghub_key.is_some() {
                let ghub_key_str = ghub_key.unwrap();
                let new_key_id = gpg::import_key(&ghub_key_str);
                write_lock_file(repo, uid, &new_key_id);
                println!("Found key for user: {} in github", uid);
                return new_key_id;
            } 
            error_out(&format!("Unable to find key for user: {}", uid));
        }
    }

    let key_id_str = key_id.unwrap();
    if gpg::has_key(&key_id_str) {
        return key_id_str;
    }

    if get_saved_key(repo, &key_id_str).is_some() {
        println!("Found key for user: {} locally", uid);
        return key_id_str;
    }

    let ghub_key = github::get_key(uid, Some(key_id_str.clone()));
    if ghub_key.is_some() {
        println!("Found key for user {} in github", uid);
        return gpg::import_key(&ghub_key.unwrap());
    }

    error_out(&format!("Unknown key id: {}, for user: {}", key_id_str, uid))
}

pub fn set_key(repo: &Repository, uid: &str, pub_key: &str) {
    save_key(repo, pub_key, uid);
}

fn write_lock_file(repo: &Repository, uid: &str, key_id: &str) {
    let path = get_lock_dir(repo).join(uid);
    fs::write(path, key_id).unwrap();
}

fn get_locked_key_id(repo: &Repository, uid: &str) -> Option<String> {
    let lock_dir = get_lock_dir(repo);
    let file_path = lock_dir.join(uid);
    if !file_path.exists() {
        return None;
    }

    let bytes = fs::read(file_path).unwrap();
    let contents = String::from_utf8(bytes).unwrap();
    return Some(contents.trim().to_string());
}

fn get_lock_dir(repo: &Repository) -> PathBuf {
    let lock_path = git_utils::get_credentials_dir(&repo).join(KEY_FOLDER_NAME).join(LOCK_FOLDER_NAME);
    if !lock_path.exists() {
        fs::create_dir_all(&lock_path).unwrap();
    }
    return lock_path;
}

fn get_saved_key(repo: &Repository, key_id: &str) -> Option<String> {
    let key_path = 
        git_utils::get_credentials_dir(repo)
        .join(KEY_FOLDER_NAME)
        .join(PUBLIC_KEY_FOLDER_NAME)
        .join(key_id);
    if !key_path.exists() || !key_path.is_file() {
        return None;
    }

    let contents = fs::read_to_string(key_path);
    if contents.is_err() {
        return None;
    }

    return Some(gpg::import_key(&contents.unwrap()));
}

fn save_key(repo: &Repository, pub_key: &str, uid: &str) {
    let key_id = gpg::import_key(pub_key);
    let path = git_utils::get_credentials_dir(repo)
        .join(KEY_FOLDER_NAME)
        .join(PUBLIC_KEY_FOLDER_NAME);
    fs::create_dir_all(path.clone()).unwrap();

    let key_path = path.join(key_id.clone());
    fs::write(key_path, pub_key).unwrap();
    write_lock_file(repo, uid, &key_id);
}