//! Throwaway repositories whose remotes are set with raw git or raw config bytes, never with Git::remote_push_urls.
use crate::support::GitRepo;
use gitio::RunOpts;
use std::fs::OpenOptions;
use std::io::Write;

/// Runs raw git in the repository; any non-zero exit panics.
pub fn raw(repo: &GitRepo, args: &[&str]) {
    repo.git.run(args, &RunOpts::default()).unwrap();
}

/// Appends `bytes` to the repository's `.git/config`, for config raw git cannot write (non-UTF-8, invalid lines).
pub fn append_config(repo: &GitRepo, bytes: &[u8]) {
    let mut config = OpenOptions::new().append(true).open(repo.path.join(".git/config")).unwrap();
    config.write_all(bytes).unwrap();
}
