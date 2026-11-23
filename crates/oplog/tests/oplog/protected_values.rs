//! Protected moves with values repeat recorded results, refuse invalid or unsafe moves, or log fast-forwards.

use crate::support::{commit, refs, set_ref, test_op, Fixture};
use gitio::Oid;
use oplog::{
    apply_protected_with_values, oplog_path, read_valued_log, ApplyOutcome, OpPhase, OplogError,
    ProtectedRef, RefMove, ValuedOutcome,
};
use std::fs;

const PROTECTED: &str = "refs/heads/protected";

#[test]
fn protected_values_repeat_returns_first_done_and_values_without_writes() {
    let f = Fixture::new();
    let root = commit(&f.git, "root", &[]);
    let child = commit(&f.git, "child", &[&root]);
    set_ref(&f.git, PROTECTED, &root);
    let policy = [ProtectedRef { refname: PROTECTED.into(), by: "release-policy".into() }];
    let mut op = test_op("repeat", "publisher", vec![RefMove {
        refname: PROTECTED.into(), old: Some(root), new: child.clone(),
    }]);
    let values = vec![("base".into(), "first".into()), ("note".into(), "recorded".into())];
    let first = apply_protected_with_values(&f.git, &f.common, &op, &policy, &values).unwrap();
    let done = match first.outcome {
        ApplyOutcome::Done(record) => record,
        other => panic!("expected Done: {other:?}"),
    };
    let bytes = fs::read(oplog_path(&f.common)).unwrap();
    op.moves[0] = RefMove {
        refname: PROTECTED.into(), old: Some(child.clone()), new: Oid::parse(&"0".repeat(40)).unwrap(),
    };
    let other_values = vec![("base".into(), "second".into())];
    assert_eq!(
        apply_protected_with_values(&f.git, &f.common, &op, &policy, &other_values).unwrap(),
        ValuedOutcome { outcome: ApplyOutcome::Repeated(done), values },
    );
    assert_eq!(fs::read(oplog_path(&f.common)).unwrap(), bytes);
    assert_eq!(f.git.read_ref(PROTECTED).unwrap(), Some(child));
}

#[test]
fn protected_values_bad_name_rewind_sideways_and_delete_are_refused_without_writes() {
    for case in ["bad_name", "rewind", "sideways", "delete"] {
        let f = Fixture::new();
        let root = commit(&f.git, "root", &[]);
        let child = commit(&f.git, "child", &[&root]);
        let sibling = commit(&f.git, "sibling", &[&root]);
        let zero = Oid::parse(&"0".repeat(40)).unwrap();
        let new = match case {
            "bad_name" | "rewind" => &root,
            "sideways" => &sibling,
            "delete" => &zero,
            _ => unreachable!(),
        };
        set_ref(&f.git, PROTECTED, &child);
        let policy = [ProtectedRef { refname: PROTECTED.into(), by: "release-policy".into() }];
        let first = if case == "bad_name" { "heads/main" } else { "refs/mortar/a" };
        let op = test_op(case, "publisher", vec![
            RefMove { refname: first.into(), old: Some(f.objects[0].clone()), new: f.objects[1].clone() },
            RefMove { refname: PROTECTED.into(), old: Some(child.clone()), new: new.clone() },
        ]);
        let before = refs(&f);
        let values = vec![("base".into(), "kept".into())];
        match (case, apply_protected_with_values(&f.git, &f.common, &op, &policy, &values).unwrap_err()) {
            ("bad_name", OplogError::BadMove { refname, .. }) => {
                assert_eq!(refname, "heads/main", "{case}");
            }
            ("rewind" | "sideways" | "delete", OplogError::Protected { refname, current, new: refused, by }) => {
                assert_eq!(refname, PROTECTED, "{case}");
                assert_eq!(current, Some(child.clone()), "{case}");
                assert_eq!(refused, *new, "{case}");
                assert_eq!(by, "release-policy", "{case}");
            }
            (_, other) => panic!("{case}: unexpected refusal {other:?}"),
        }
        assert!(!oplog_path(&f.common).exists(), "{case}: wrote an intent");
        assert!(read_valued_log(&f.common).unwrap().records.is_empty(), "{case}");
        assert_eq!(refs(&f), before, "{case}");
        assert_eq!(f.git.read_ref(PROTECTED).unwrap(), Some(child), "{case}");
    }
}

#[test]
fn protected_values_fast_forward_moves_ref_and_records_values() {
    let f = Fixture::new();
    let root = commit(&f.git, "root", &[]);
    let child = commit(&f.git, "child", &[&root]);
    set_ref(&f.git, PROTECTED, &root);
    let policy = [ProtectedRef { refname: PROTECTED.into(), by: "release-policy".into() }];
    let op = test_op("fast-forward", "publisher", vec![RefMove {
        refname: PROTECTED.into(), old: Some(root), new: child.clone(),
    }]);
    let values = vec![
        ("base".into(), "first".into()),
        ("note".into(), "say \"hi\"\nline two".into()),
        ("base".into(), "".into()),
    ];
    let got = apply_protected_with_values(&f.git, &f.common, &op, &policy, &values).unwrap();
    let done = match got.outcome {
        ApplyOutcome::Done(record) => record,
        other => panic!("expected Done: {other:?}"),
    };
    assert_eq!(got.values, values);
    assert_eq!(f.git.read_ref(PROTECTED).unwrap(), Some(child));
    let log = read_valued_log(&f.common).unwrap();
    assert!(log.torn_lines.is_empty());
    assert_eq!(log.records.len(), 2);
    assert_eq!(log.records[0].record.phase, OpPhase::Intent);
    assert_eq!(log.records[1].record.phase, OpPhase::Done);
    assert_eq!(log.records[1].record, done);
    assert_eq!(log.records[0].values, values);
    assert_eq!(log.records[1].values, values);
}
