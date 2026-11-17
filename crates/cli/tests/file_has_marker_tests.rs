//! file_has_marker: true for a regular file (or a link to one) whose bytes hold the marker, wherever it falls in the
//! 64 KiB reads; false for a file without it, a directory or a missing path. On files each test makes in its own
//! scratch folder under the system temp dir.
//! ZIT (rule 11, S-104): it calls cli::path::file_has_marker on real files, directories and symlinks it makes itself
//! in that folder and removes after the test.

use std::fs;
use std::os::unix::fs::symlink;
use std::path::PathBuf;

use cli::path::file_has_marker;

const MARKER: &[u8] = b"\0test-marker/file_has_marker\0";

/// A scratch folder unique to one test in this process, removed when the test ends.
struct Scratch(PathBuf);

impl Scratch {
    fn new(test: &str) -> Scratch {
        let root = std::env::temp_dir().join(format!("cli-file-has-marker-{}-{test}", std::process::id()));
        fs::create_dir_all(&root).unwrap();
        Scratch(root)
    }

    fn file(&self, name: &str, bytes: &[u8]) -> PathBuf {
        let path = self.0.join(name);
        fs::write(&path, bytes).unwrap();
        path
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn a_file_holding_the_marker_has_it() {
    let scratch = Scratch::new("holding");
    let path = scratch.file("mr", &[b"head ".as_slice(), MARKER, b" tail"].concat());
    assert!(file_has_marker(&path, MARKER));
}

#[test]
fn a_file_without_the_marker_does_not_have_it() {
    let scratch = Scratch::new("without");
    let path = scratch.file("mr", b"#!/bin/sh\nexec myrepos \"$@\"\n");
    assert!(!file_has_marker(&path, MARKER));
}

#[test]
fn the_marker_straddling_the_64_kib_boundary_is_found() {
    let scratch = Scratch::new("straddling");
    let path = scratch.file("mr", &[vec![b'x'; 65536 - 3], MARKER.to_vec(), vec![b'y'; 100]].concat());
    assert!(file_has_marker(&path, MARKER));
}

#[test]
fn a_file_shorter_than_one_read_with_the_marker_near_the_start_has_it() {
    let scratch = Scratch::new("short");
    let path = scratch.file("mr", &[b"ab".as_slice(), MARKER].concat());
    assert!(file_has_marker(&path, MARKER));
}

#[test]
fn a_directory_does_not_have_it() {
    let scratch = Scratch::new("directory");
    let dir = scratch.0.join("mr");
    fs::create_dir(&dir).unwrap();
    assert!(!file_has_marker(&dir, MARKER));
}

#[test]
fn a_missing_path_does_not_have_it() {
    let scratch = Scratch::new("missing");
    assert!(!file_has_marker(&scratch.0.join("gone"), MARKER));
}

#[test]
fn a_symlink_to_a_file_holding_the_marker_has_it() {
    let scratch = Scratch::new("symlink");
    let real = scratch.file("mortar", &[b"head ".as_slice(), MARKER].concat());
    let link = scratch.0.join("mr");
    symlink(&real, &link).unwrap();
    assert!(file_has_marker(&link, MARKER));
}
