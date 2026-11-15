//! classify_conflicted: a blob is written, no entry is absent, and a commit refusal naming the file is a gitlink,
//! including for a quoted or spaced path. Every other refusal goes through gitio's From, so its cases are in
//! tests/join_rit/classify.rs (S-101), except a commit naming another path, which resolve cannot reach (S-109).

use crate::classify_conflicted::classify_conflicted;
use crate::conflicted_entry::ConflictedEntry;
use gitio::{EntryKind, Oid, TreeEntryError};
use std::path::{Path, PathBuf};

fn not_a_blob(path: &str, kind: EntryKind) -> TreeEntryError {
    TreeEntryError::NotABlob { path: PathBuf::from(path), kind }
}

#[test]
fn a_blob_is_written() {
    let blob = Oid::parse("1111111111111111111111111111111111111111").unwrap();
    assert_eq!(classify_conflicted(Path::new("src/a.rs"), Ok(Some(blob))), Ok(ConflictedEntry::Write));
}

#[test]
fn no_entry_is_absent() {
    assert_eq!(classify_conflicted(Path::new("old/name.rs"), Ok(None)), Ok(ConflictedEntry::Absent));
}

#[test]
fn a_commit_naming_the_file_is_a_gitlink() {
    let entry = Err(not_a_blob("vendor/lib", EntryKind::Commit));
    assert_eq!(classify_conflicted(Path::new("vendor/lib"), entry), Ok(ConflictedEntry::Gitlink));
}

#[test]
fn a_path_with_a_space_and_quotes_is_a_gitlink() {
    let entry = Err(not_a_blob("my \"odd\" sub", EntryKind::Commit));
    assert_eq!(classify_conflicted(Path::new("my \"odd\" sub"), entry), Ok(ConflictedEntry::Gitlink));
}
