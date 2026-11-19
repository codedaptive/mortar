//! Tests the public join::paused_join with read_join_state, decode_join_state and join_state_path for real; touches
//! only its own temp files, no git.
use gitio::Oid;
use join::{paused_join, JoinRunError, PausedJoin};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::{env, fs, process};

/// A fresh directory under the temp dir standing in for a folder's git dir; removed on drop, panic included.
struct TempDir(PathBuf);

impl TempDir {
    fn new(test: &str) -> TempDir {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        let dir = env::temp_dir().join(format!("mortar-join-zit-{}-{test}-{n}", process::id()));
        fs::create_dir_all(&dir).unwrap();
        TempDir(dir)
    }

    fn path(&self) -> &Path {
        &self.0
    }

    fn state(&self) -> PathBuf {
        self.0.join("mortar-join.json")
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn no_state_file_answers_none() {
    // Joins paused_join -> join_state_path -> read_join_state's NotFound arm.
    let dir = TempDir::new("no_state_file_answers_none");
    let result = paused_join(dir.path());
    assert!(matches!(&result, Ok(None)), "{result:?}");
}

#[test]
fn a_valid_state_answers_its_commit_and_this_rounds_files() {
    // Joins paused_join -> read_join_state -> decode_join_state on a hand-written state, then the PausedJoin copy.
    let dir = TempDir::new("a_valid_state_answers_its_commit_and_this_rounds_files");
    let text = concat!(
        "{\"request_ref\":\"refs/heads/request/r1\",",
        "\"request_tip\":\"1111111111111111111111111111111111111111\",",
        "\"onto\":\"2222222222222222222222222222222222222222\",",
        "\"replayed_tip\":\"3333333333333333333333333333333333333333\",",
        "\"commit\":\"4444444444444444444444444444444444444444\",",
        "\"tree\":\"5555555555555555555555555555555555555555\",",
        "\"remaining\":[],\"files\":[\"src/a.rs\"],\"all_files\":[\"old/c.md\",\"src/a.rs\"],",
        "\"files_written\":false,\"rebased_to\":null,\"folder_tree\":null}\n",
    );
    fs::write(dir.state(), text).unwrap();
    let expected = PausedJoin {
        commit: Oid::parse("4444444444444444444444444444444444444444").unwrap(),
        files: vec![PathBuf::from("src/a.rs")],
    };
    assert_eq!(paused_join(dir.path()).unwrap(), Some(expected));
}

#[test]
fn a_corrupt_state_is_state_corrupt_and_kept() {
    // Joins paused_join -> read_join_state -> decode_join_state's refusal, mapped to StateCorrupt naming the file.
    let dir = TempDir::new("a_corrupt_state_is_state_corrupt_and_kept");
    let bytes = b"{\"request_ref\":1}\n";
    fs::write(dir.state(), bytes).unwrap();
    let result = paused_join(dir.path());
    assert!(matches!(&result, Err(JoinRunError::StateCorrupt { path, .. }) if *path == dir.state()), "{result:?}");
    assert_eq!(fs::read(dir.state()).unwrap(), bytes);
}

#[test]
fn a_non_utf8_state_is_state_read() {
    // Joins paused_join -> read_join_state's read_to_string failure on bytes that are not UTF-8.
    let dir = TempDir::new("a_non_utf8_state_is_state_read");
    fs::write(dir.state(), [0xff, 0xfe, b'{']).unwrap();
    let result = paused_join(dir.path());
    assert!(matches!(&result, Err(JoinRunError::StateRead { path, .. }) if *path == dir.state()), "{result:?}");
}

#[test]
fn a_directory_in_place_of_the_state_is_state_read() {
    // Joins paused_join -> read_join_state's read_to_string failure on a directory at the state path.
    let dir = TempDir::new("a_directory_in_place_of_the_state_is_state_read");
    fs::create_dir(dir.state()).unwrap();
    let result = paused_join(dir.path());
    assert!(matches!(&result, Err(JoinRunError::StateRead { path, .. }) if *path == dir.state()), "{result:?}");
}
