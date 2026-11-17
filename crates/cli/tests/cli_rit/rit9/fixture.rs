//! RIT9's child invocation and independent observations over the existing disposable real-crate World.
//! No second process/Git fixture: reuse RIT8's records, repository ownership and exact index/file observation.
pub use crate::rit8::fixture::World;
use crate::{json::{check, Report}, support::raw};
use std::{fs, path::{Path, PathBuf}};

pub fn run(w: &World, cwd: &Path, words: &[&str], exit: i32, outcome: &str) -> Report {
    let mut command = w.s.f.command(cwd, &["--json", "--actor", "rit9"]);
    command.args(words);
    check(command.output().unwrap(), words[0], exit, outcome)
}
pub fn app(w: &World, words: &[&str], exit: i32, outcome: &str) -> Report {
    run(w, &w.s.f.git.repo, words, exit, outcome)
}
#[derive(Debug, PartialEq, Eq)]
pub struct Observation {
    store: Vec<Vec<u8>>,
    #[allow(clippy::type_complexity)] // why: one field's shape, spelled out here; an alias would touch more RIT lines (S-130)
    folders: Vec<(PathBuf, Vec<u8>, Vec<u8>, Vec<(PathBuf, Vec<u8>)>)>,
}
/// Every registered non-bare worktree, its actual record, all staged entries and every file (untracked included).
/// The store includes every ref, op-log bytes, remote refs, registrations and landing state, not just HEAD.
pub fn observe(w: &World) -> Observation {
    let folders = w.s.f.git.worktree_list().unwrap().into_iter().filter(|t| !t.bare && !t.prunable).map(|t| {
        let dir = w.s.f.git.git_dir(&t.path).unwrap();
        let record = fs::read(folders::record::record_path(&dir)).unwrap_or_default();
        let (index, files) = w.folder_state(&t.path);
        (t.path, record, index, files)
    }).collect();
    Observation { store: w.store(), folders }
}
pub fn saved(w: &World, path: &Path, sentence: &str) {
    check(w.s.f.command(path, &["--json", "save", sentence]).output().unwrap(), "save", 0, "saved");
}
pub fn bytes_in_tree(path: &Path) -> u64 {
    let metadata = fs::symlink_metadata(path).unwrap();
    if !metadata.is_dir() { return metadata.len(); }
    fs::read_dir(path).unwrap().map(|e| bytes_in_tree(&e.unwrap().path())).sum()
}
pub fn ref_at(w: &World, name: &str) -> Option<gitio::Oid> { w.s.f.git.read_ref(name).unwrap() }
pub fn refs(w: &World) -> Vec<u8> { raw(&w.s.f.git, &["for-each-ref", "--format=%(refname) %(objectname)"]) }
