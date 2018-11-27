extern crate git2;
extern crate reqwest;

mod git_utils;
mod git_config;
mod command;
mod gpg;

use git2::Repository;

fn main() {
    let repo_res = Repository::discover(".");

    if repo_res.is_err() {
        panic!("Current directory isn't a git repo!");
    }

    let repo = repo_res.unwrap();

    println!("{}", gpg::has_key("linuxbash8"));
}