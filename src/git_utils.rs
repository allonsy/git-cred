use std::path::Path;
use std::path::PathBuf;
use git2::Repository;
use git_config;
use std::fs;

pub fn get_top_level_directory(repo: &Repository) -> &Path {
    let path = repo.path();
    if path.ends_with(".git") {
        return path.parent().unwrap();
    } else {
        return path;
    }
}

pub fn get_credentials_dir_path(repo: &Repository) -> PathBuf {
    let mut path = get_top_level_directory(repo).to_path_buf();
    path.push(".credential-store");
    return path;
}

/* Creates credential store if it doesn't exist. Returns the absolute path to the credential store */
pub fn get_credentials_dir(repo: &Repository) -> PathBuf {
    let possible_credentials_location = git_config::get_credentials_location(repo);
    let location = if possible_credentials_location.is_none() {
        get_credentials_dir_path(repo)
    } else {
        let mut top_level = get_top_level_directory(repo).to_path_buf();
        top_level.push(possible_credentials_location.unwrap());
        top_level
    };
    if !location.exists() {
        println!("Creating new credential store at: {}", location.display());
        fs::create_dir(location.clone()).unwrap();
    }
    return location;
}

pub fn get_gpg_ids_for_file(repo: &Repository, path: &str) -> Vec<String> {
    let mut cred_path = get_credentials_dir(repo);
    cred_path = cred_path.join(path);
    Vec::new()

}

fn is_parent_or_equal(path1: &Path, path2: &Path) -> bool {
    let p1parts: Vec<std::path::Components> = path1.components().collect();
    let p2parts: Vec<std::path::Components> = path2.components().collect();

    if p2parts.len() < p1parts.len() {
        return false;
    }

    for i in 0..p1parts.len() {
        if p1parts[i] != p2parts[i] {
            return false;
        }
    }
    
    return true;
}