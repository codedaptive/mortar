//! S-14: joins apply's pre-intent ref validation, real git grammar and durable-log absence.

use crate::support::{operation, refs, Fixture};
use gitio::RunOpts;
use oplog::{apply, oplog_path, read_op_log, OplogError};

#[test]
fn apply_refuses_a_name_outside_refs_before_writing_an_intent() {
    let f = Fixture::new();
    // git check-ref-format accepts this spelling, but it is outside show-ref's readable namespace.
    let accepted = f.git.run(&["check-ref-format", "heads/not-a-full-ref"], &RunOpts::default()).unwrap();
    assert_eq!(accepted.status, 0);
    assert_bad_name(&f, "heads/not-a-full-ref");
    assert!(!f.common.join("heads/not-a-full-ref").exists());
}

#[test]
fn apply_refuses_a_name_git_check_ref_format_rejects_before_writing_an_intent() {
    let f = Fixture::new();
    let rejected = f.git.run(&["check-ref-format", "refs/heads/bad..name"], &RunOpts::default()).unwrap_err();
    assert_eq!(rejected.code, Some(1));
    assert_bad_name(&f, "refs/heads/bad..name");
}

fn assert_bad_name(f: &Fixture, name: &str) {
    let mut op = operation(&f.objects);
    // Include a valid move before the bad one; rejecting late must not move it or leave an intent.
    op.moves[1].refname = name.into();
    let before = refs(f);
    assert!(!oplog_path(&f.common).exists());
    match apply(&f.git, &f.common, &op).unwrap_err() {
        OplogError::BadMove { refname, reason } => {
            assert_eq!(refname, name);
            assert!(!reason.is_empty());
        }
        other => panic!("expected pre-intent BadMove for {name}: {other:?}"),
    }
    assert!(!oplog_path(&f.common).exists(), "invalid name wrote an intent");
    assert!(read_op_log(&f.common).unwrap().records.is_empty());
    assert_eq!(refs(f), before, "invalid name moved refs");
}
