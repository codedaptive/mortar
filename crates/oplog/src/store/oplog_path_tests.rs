//! oplog_path: the log sits in the `mortar` folder of the common dir.

use std::path::{Path, PathBuf};

use super::oplog_path;

#[test]
fn log_is_mortar_oplog_jsonl_under_common() {
    assert_eq!(
        oplog_path(Path::new("/repo/.git")),
        PathBuf::from("/repo/.git/mortar/oplog.jsonl")
    );
}

#[test]
fn relative_common_stays_relative() {
    assert_eq!(
        oplog_path(Path::new(".git")),
        PathBuf::from(".git/mortar/oplog.jsonl")
    );
}
