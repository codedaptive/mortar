//! One attempt a repository holds, as attempts_of reads it.
//! A bare type; attempts_of_tests and keep_plan_tests exercise it.

use std::path::PathBuf;

use gitio::Oid;

/// Attempt `k`, on `branch` (the full name, `refs/heads/try/<zone>/<request>/<k>`) at `tip`; `folder` is the
/// worktree that has it checked out, None when none has.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Attempt {
    pub k: u32,
    pub branch: String,
    pub tip: Oid,
    pub folder: Option<PathBuf>,
}
