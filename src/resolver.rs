use git_utils;
use gpg;
use git2::Repository;
use std::path::PathBuf;
use std::fs;
use util::error_out;

const KEY_FOLDER_NAME: &'static str = ".keys";
const LOCK_FOLDER_NAME: &'static str = "locks";

pub fn resolve_name(repo: &Repository, uid: &str) -> String {
    let key_id = get_locked_key_id(repo, uid);
    if !key_id.is_none() {
        if gpg::has_key(uid) {
            let new_key_id = gpg::get_key_id(uid);
            write_lock_file(repo, uid, &new_key_id);
        } else {
            error_out(&format!("Unable to find key for user: {}", uid));
        }
    }

    let key_id_str = key_id.unwrap();
    if gpg::has_key(&key_id_str) {
        return key_id_str;
    }

    error_out(&format!("Unknown key id: {}, for user: {}", key_id_str, uid))
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