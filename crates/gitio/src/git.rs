//! Git: which git binary to run, against which repository, with which extra environment.

use std::ffi::OsString;
use std::path::PathBuf;

/// One repository's git: every run passes `repo` as `-C` (unless `opts.cwd` replaces it) and adds `env` to the child's environment.
#[derive(Clone, Debug)]
pub struct Git {
    /// The git binary; a bare name is looked up on PATH when the process starts.
    pub program: PathBuf,
    /// The repository directory, passed to git as `-C`.
    pub repo: PathBuf,
    /// Extra environment for every run, applied after gitio's own settings.
    pub env: Vec<(OsString, OsString)>,
}
