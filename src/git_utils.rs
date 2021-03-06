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
        create_default_gpg_id(repo, &location);
    }
    return location;
}

fn create_default_gpg_id(repo: &Repository, path: &Path) {
    let default_email = git_config::get_email(repo);
    let string_to_write = if default_email.is_some() {
        let mut default_email_res = default_email.unwrap();
        println!("Using default gpg id: {}", default_email_res);
        default_email_res += "\n";
        default_email_res
    } else {
        String::new()
    };

    let gpg_path = path.to_path_buf().join(".gpg_id");
    fs::write(gpg_path, string_to_write).unwrap();
}

pub fn create_sub_dir(repo: &Repository, path: &Path, recipients: Vec<String>) {
    let actual_path = get_credentials_dir_path(repo).join(path);
    fs::create_dir_all(actual_path.clone()).unwrap();

    let mut string_to_write = String::new();
    if recipients.is_empty() {
        string_to_write = git_config::get_email(repo).unwrap_or(String::new());
        string_to_write += "\n";
    } else {
        for recipient in recipients {
            string_to_write = string_to_write + &recipient + "\n";
        }
    }
    let gpg_path = actual_path.join(".gpg_id");
    fs::write(gpg_path, string_to_write).unwrap();
}