use git2::Repository;
use std::path::PathBuf;

const LOCATION_KEY_NAME: &'static str = "creds.location";

pub fn get_credentials_location(repo: &Repository) -> Option<PathBuf> {
    let config = repo.config().unwrap();

    let possible_location = config.get_path(LOCATION_KEY_NAME);
    if possible_location.is_err() {
        return None;
    }
    return Some(possible_location.unwrap());
}