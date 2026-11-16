//! Joins check_restore_ref_names -> check_ref_names -> real git check-ref-format, and restore's refusal of a bad
//! ref name before any ref is read or anything is written.

use crate::support::{caller, refs, test_op, Fixture};
use oplog::{apply, check_restore_ref_names, oplog_path, restore, OplogError, RefMove};
use std::fs;

#[test]
fn a_well_formed_full_name_passes() {
    let f = Fixture::new();
    let checked = check_restore_ref_names(&f.git, &["refs/heads/main".into()]);
    assert!(checked.is_ok(), "{checked:?}");
}

#[test]
fn a_name_git_check_ref_format_refuses_is_bad_move() {
    let f = Fixture::new();
    assert_bad_move(check_restore_ref_names(&f.git, &["refs/heads/a..b".into()]), "refs/heads/a..b");
}

#[test]
fn the_first_refused_name_after_a_good_one_is_named() {
    let f = Fixture::new();
    let names = ["refs/heads/main".to_string(), "refs/heads/a..b".to_string()];
    assert_bad_move(check_restore_ref_names(&f.git, &names), "refs/heads/a..b");
}

#[test]
fn restore_refuses_a_refused_name_with_the_log_and_refs_unchanged() {
    let f = Fixture::new();
    let target = test_op("target", "original-agent", vec![RefMove {
        refname: "refs/mortar/a".into(),
        old: Some(f.objects[0].clone()),
        new: f.objects[1].clone(),
    }]);
    apply(&f.git, &f.common, &target).unwrap();
    let log = fs::read(oplog_path(&f.common)).unwrap();
    let before = refs(&f);
    let refused = restore(&f.git, &f.common, "target", &["refs/heads/a..b".into()], &caller("bad-restore"), &[]);
    assert_bad_move(refused.map(|_| ()), "refs/heads/a..b");
    assert_eq!(fs::read(oplog_path(&f.common)).unwrap(), log, "a refused restore wrote to the op log");
    assert_eq!(refs(&f), before, "a refused restore moved refs");
}

fn assert_bad_move(result: Result<(), OplogError>, name: &str) {
    match result {
        Err(OplogError::BadMove { refname, reason }) => {
            assert_eq!(refname, name);
            assert!(!reason.is_empty());
        }
        other => panic!("expected BadMove naming {name}: {other:?}"),
    }
}
