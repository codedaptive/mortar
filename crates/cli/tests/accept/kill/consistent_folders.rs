//! 03's "every request and attempt folder's HEAD and index match its branch": no half-merged folder. Real
//! processes: kill_consistent.rs reads those folders after every case on every drive.

use std::path::{Path, PathBuf};

use crate::stage::git_out;
use crate::world::World;

/// Whether every folder in `folders` (the request and attempt folders) holds HEAD on a branch, HEAD at that
/// branch's tip, and a clean `git status --porcelain` (index and files at HEAD), and what each one showed.
pub fn consistent_folders(world: &World, folders: &[PathBuf]) -> (bool, String) {
    let seen: Vec<(bool, String)> = folders.iter().map(|folder| one(world, folder)).collect();
    let ok = seen.iter().all(|(ok, _)| *ok);
    (ok, seen.into_iter().map(|(_, text)| text).collect::<Vec<_>>().join("; "))
}

fn one(world: &World, folder: &Path) -> (bool, String) {
    let name = folder.file_name().map(|name| name.to_string_lossy().into_owned()).unwrap_or_default();
    let branch = git_out(world, folder, &["symbolic-ref", "-q", "HEAD"]);
    let head = git_out(world, folder, &["rev-parse", "-q", "--verify", "HEAD"]);
    let tip = if branch.is_empty() { String::new() } else { git_out(world, &world.work, &["rev-parse", &branch]) };
    let status = git_out(world, folder, &["status", "--porcelain"]);
    let ok = !branch.is_empty() && !head.is_empty() && head == tip && status.is_empty();
    let changes = match status.lines().next() {
        None => "clean".to_string(),
        Some(first) => format!("{} status line(s), first {first:?}", status.lines().count()),
    };
    (ok, format!("{name}: HEAD {head} on {branch} at {tip}, {changes}"))
}
