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