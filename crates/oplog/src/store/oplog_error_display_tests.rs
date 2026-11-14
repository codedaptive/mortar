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
        reason: "the record stops before its JSON is complete".to_string(),
    };
    assert_eq!(
        err.to_string(),
        "malformed op data: the record stops before its JSON is complete; \
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
         nothing was changed; find who moved them with mortar log --ref <ref>, then close the op by hand \
         with a done or failed record in the op log, since recover refuses every command until it is closed"
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

#[test]
fn protected_delete_names_the_ref_its_value_and_the_protector() {
    let err = OplogError::Protected {
        refname: "refs/heads/main".to_string(),
        current: Some(oid("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")),
        new: oid("0000000000000000000000000000000000000000"),
        by: "land".to_string(),
    };
    assert_eq!(
        err.to_string(),
        "ref refs/heads/main is protected by land: deleting it is refused (it holds \
         aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa); nothing was changed; ask land before removing it"
    );
}

#[test]
fn protected_move_names_both_values_and_says_fast_forward_only() {
    let err = OplogError::Protected {
        refname: "refs/heads/main".to_string(),
        current: Some(oid("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")),
        new: oid("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"),
        by: "land".to_string(),
    };
    assert_eq!(
        err.to_string(),
        "ref refs/heads/main is protected by land: moving it from aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa to \
         bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb is not a fast-forward; nothing was changed; move it only to a \
         commit that has aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa as an ancestor, or ask land"
    );
}

#[test]
fn nothing_to_undo_joins_the_refs() {
    let err = OplogError::NothingToUndo {
        refs: vec!["refs/heads/a".to_string(), "refs/heads/b".to_string()],
    };
    assert_eq!(
        err.to_string(),
        "nothing to undo: no done op in the op log moved refs/heads/a, refs/heads/b; check the ref names with \
         mortar log --ref <ref>"
    );
}

#[test]
fn moved_since_by_a_logged_op_names_that_op_and_its_actor() {
    let err = OplogError::MovedSince {
        op_id: "op-1".to_string(),
        refname: "refs/heads/a".to_string(),
        left: Some(oid("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")),
        actual: Some(oid("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb")),
        moved_by: Some(("op-2".to_string(), "simms".to_string())),
    };
    assert_eq!(
        err.to_string(),
        "cannot undo op op-1: ref refs/heads/a moved since (that op left it at \
         aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa, it now holds bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb), by op \
         op-2 of simms; undo op op-2 first, or restore the ref to the value you want"
    );
}

#[test]
fn moved_since_outside_mortar_says_no_logged_op_explains_it() {
    let err = OplogError::MovedSince {
        op_id: "op-1".to_string(),
        refname: "refs/heads/a".to_string(),
        left: None,
        actual: Some(oid("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb")),
        moved_by: None,
    };
    assert_eq!(
        err.to_string(),
        "cannot undo op op-1: ref refs/heads/a moved since (that op left it at absent, it now holds \
         bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb) outside mortar: no logged op explains it; find who moved it, \
         then restore the ref to the value you want"
    );
}

#[test]
fn unknown_op_names_the_op_and_says_find_it_with_mortar_log() {
    let err = OplogError::UnknownOp {
        op_id: "op-404".to_string(),
    };
    assert_eq!(
        err.to_string(),
        "cannot restore: op op-404 is not in the op log; find the op id with mortar log"
    );
}

#[test]
fn value_unknown_names_the_ref_and_the_op() {
    let err = OplogError::ValueUnknown {
        op_id: "op-3".to_string(),
        refname: "refs/heads/x".to_string(),
    };
    assert_eq!(
        err.to_string(),
        "cannot restore ref refs/heads/x as of op op-3: no done op in the op log moved it, so its value then is \
         unknown; name only refs mortar has moved"
    );
}

#[test]
fn not_open_names_the_op_and_says_find_it_in_the_log_file() {
    let err = OplogError::NotOpen {
        op_id: "op-9".to_string(),
    };
    assert_eq!(
        err.to_string(),
        "cannot close op op-9: it has no open intent in the op log (never written, or already closed); nothing \
         was written; in mortar/oplog.jsonl, find the intent line that no later done or failed line closes and \
         use its op_id"
    );
}
