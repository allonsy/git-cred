use gpg;
use git_utils;
use git2::Repository;
use std::path::Path;
use util;


pub fn decrypt(repo: &Repository, path: &Path) -> String {
    let credential_path = git_utils::get_credentials_dir(repo);
    let path_to_file = credential_path.join(path);

    let contents = gpg::decrypt(path_to_file);
    if contents.is_err() {
        util::error_out(&format!("Decryption failed for file: {}", path.display()));
    }
    return contents.unwrap();
}