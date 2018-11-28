use gpg;
use git_utils;
use git2::Repository;
use std::path::Path;


pub fn decrypt(repo: &Repository, path: &Path) -> String {
    let credential_path = git_utils::get_credentials_dir(repo);
    let path_to_file = credential_path.join(path);

    gpg::decrypt(path_to_file).unwrap()
}