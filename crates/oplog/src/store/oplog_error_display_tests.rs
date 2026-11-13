//! Refusal texts: each variant names its line, path, op or refs, and says what to do.

use std::path::PathBuf;

use gitio::{GitError, Oid};

use super::{OplogError, RefMove};

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

#[test]
fn malformed_line_names_the_line_and_says_repair() {
    let err = OplogError::Malformed {
        line: Some(7),
        reason: "missing key \"at\"".to_string(),
    };
    assert_eq!(
        err.to_string(),
        "op log line 7 is not a record (missing key \"at\"); mortar did not write it, so check what did, \
         then repair or remove that line of mortar/oplog.jsonl"
    );
}

#[test]
fn malformed_without_line_names_the_reason_and_says_check() {
    let err = OplogError::Malformed {
        line: None,
        reason: "git rev-parse printed \"HEAD\", which is not a git object id".to_string(),
    };
    assert_eq!(
        err.to_string(),
        "malformed op data: git rev-parse printed \"HEAD\", which is not a git object id; \
         mortar did not write this text, so check what did"
    );
}

#[test]
fn io_names_the_path_and_the_cause() {
    let err = OplogError::Io {
        path: PathBuf::from("/repo/.git/mortar/oplog.jsonl"),
        source: std::io::Error::new(std::io::ErrorKind::PermissionDenied, "denied here"),
    };
    assert_eq!(
        err.to_string(),
        "/repo/.git/mortar/oplog.jsonl: denied here; check that path's folder and its permissions, then run again"
    );
}

#[test]
fn git_shows_git_error_and_says_fix() {
    let err = OplogError::Git(GitError {
        args: vec!["rev-parse".to_string(), "-q".to_string()],
        code: Some(128),
        stderr: "fatal: not a git repository\n".to_string(),
    });
    assert_eq!(
        err.to_string(),
        "git rev-parse -q: exit code 128: fatal: not a git repository; fix what git reports, then run again"
    );
}

#[test]
fn mixed_names_op_key_and_each_ref_with_its_values() {
    let err = OplogError::Mixed {
        op_id: "op-9".to_string(),
        key: "land:zone-a".to_string(),
        refs: vec![
            (
                RefMove {
                    refname: "refs/heads/a".to_string(),
                    old: None,
                    new: oid("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"),
                },
                Some(oid("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")),
            ),
            (
                RefMove {
                    refname: "refs/heads/b".to_string(),
                    old: Some(oid("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb")),
                    new: oid("cccccccccccccccccccccccccccccccccccccccc"),
                },
                None,
            ),
        ],
    };
    assert_eq!(
        err.to_string(),
        "op op-9 (key land:zone-a) is open and its refs are neither all at their new values nor all at their \
         old ones: \
         refs/heads/a planned absent -> aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa, now aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa; \
         refs/heads/b planned bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb -> cccccccccccccccccccccccccccccccccccccccc, now absent; \
         nothing was changed; find who moved them with history by ref, then close the op by hand with a done \
         or failed record in the op log, since recover refuses every command until it is closed"
    );
}

#[test]
fn bad_move_names_the_quoted_ref_and_the_reason() {
    let err = OplogError::BadMove {
        refname: "refs/heads/a b".to_string(),
        reason: "is not a valid ref name".to_string(),
    };
    assert_eq!(
        err.to_string(),
        "op cannot be applied: ref \"refs/heads/a b\" is not a valid ref name; fix the op's moves and run again"
    );
}

#[test]
fn bad_move_shows_an_empty_ref_name_as_empty_quotes() {
    let err = OplogError::BadMove {
        refname: String::new(),
        reason: "is empty".to_string(),
    };
    assert_eq!(
        err.to_string(),
        "op cannot be applied: ref \"\" is empty; fix the op's moves and run again"
    );
}
