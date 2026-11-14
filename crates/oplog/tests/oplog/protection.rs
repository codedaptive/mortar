//! S-29: joins real commit ancestry -> protected-move checks -> apply/undo/restore and durable log effects.

use crate::support::{caller, commit, refs, set_ref, test_op, Fixture};
use gitio::Oid;
use oplog::{apply_protected, oplog_path, read_op_log, restore, undo, ApplyOutcome, OpPhase, OplogError, ProtectedRef, RefMove};
use std::fs;

const PROTECTED: &str = "refs/heads/protected";

#[test]
fn protected_create_and_fast_forward_apply_while_unprotected_refs_can_rewind() {
    let f = Fixture::new();
    let root = commit(&f.git, "root", &[]);
    let child = commit(&f.git, "child", &[&root]);
    let policy = [ProtectedRef { refname: PROTECTED.into(), by: "release-policy".into() }];
    let create = test_op("create", "publisher", vec![
        RefMove { refname: PROTECTED.into(), old: None, new: root.clone() },
    ]);
    let created = match apply_protected(&f.git, &f.common, &create, &policy).unwrap() {
        ApplyOutcome::Done(record) => record,
        other => panic!("creation must be allowed: {other:?}"),
    };
    assert_eq!(f.git.read_ref(PROTECTED).unwrap(), Some(root.clone()));
    let forward = test_op("fast-forward", "publisher", vec![
        RefMove { refname: PROTECTED.into(), old: Some(root.clone()), new: child.clone() },
        RefMove { refname: "refs/mortar/a".into(), old: Some(f.objects[0].clone()), new: f.objects[2].clone() },
    ]);
    assert!(matches!(apply_protected(&f.git, &f.common, &forward, &policy).unwrap(), ApplyOutcome::Done(_)));
    assert_eq!(f.git.read_ref(PROTECTED).unwrap(), Some(child.clone()));
    assert_eq!(f.git.read_ref("refs/mortar/a").unwrap(), Some(f.objects[2].clone()));
    let unprotected = test_op("unprotected-rewind", "publisher", vec![
        RefMove { refname: "refs/mortar/a".into(), old: Some(f.objects[2].clone()), new: f.objects[0].clone() },
    ]);
    assert!(matches!(apply_protected(&f.git, &f.common, &unprotected, &policy).unwrap(), ApplyOutcome::Done(_)));
    assert_eq!(f.git.read_ref("refs/mortar/a").unwrap(), Some(f.objects[0].clone()));
    let log = read_op_log(&f.common).unwrap();
    assert!(log.torn_lines.is_empty());
    assert_eq!(log.records.len(), 6);
    assert_eq!(log.records.iter().map(|r| r.phase.clone()).collect::<Vec<_>>(), vec![
        OpPhase::Intent, OpPhase::Done, OpPhase::Intent, OpPhase::Done, OpPhase::Intent, OpPhase::Done,
    ]);
    assert_eq!(log.records[1], created);
    assert_eq!(log.records[3].op_id, "fast-forward");
    assert_eq!(log.records[5].op_id, "unprotected-rewind");

    // A repeated done key wins even if its new arguments would now try to delete the protected ref.
    let mut repeated = create;
    repeated.moves[0] = RefMove {
        refname: PROTECTED.into(), old: Some(child.clone()), new: Oid::parse(&"0".repeat(40)).unwrap(),
    };
    let bytes = fs::read(oplog_path(&f.common)).unwrap();
    assert_eq!(apply_protected(&f.git, &f.common, &repeated, &policy).unwrap(), ApplyOutcome::Repeated(created));
    assert_eq!(fs::read(oplog_path(&f.common)).unwrap(), bytes);
    assert_eq!(f.git.read_ref(PROTECTED).unwrap(), Some(child));
}

#[test]
fn protected_rewind_sideways_unrelated_and_delete_are_refused_before_any_intent() {
    for case in ["rewind", "sideways", "unrelated", "delete"] {
        let f = Fixture::new();
        let root = commit(&f.git, "root", &[]);
        let child = commit(&f.git, "child", &[&root]);
        let sibling = commit(&f.git, "sibling", &[&root]);
        let unrelated = commit(&f.git, "unrelated", &[]);
        let zero = Oid::parse(&"0".repeat(40)).unwrap();
        let new = match case {
            "rewind" => &root,
            "sideways" => &sibling,
            "unrelated" => &unrelated,
            "delete" => &zero,
            _ => unreachable!(),
        };
        set_ref(&f.git, PROTECTED, &child);
        let policy = [ProtectedRef { refname: PROTECTED.into(), by: "release-policy".into() }];
        let op = test_op(case, "publisher", vec![
            // A refused protected second move must not allow an earlier unprotected move through.
            RefMove { refname: "refs/mortar/a".into(), old: Some(f.objects[0].clone()), new: f.objects[1].clone() },
            RefMove { refname: PROTECTED.into(), old: Some(child.clone()), new: new.clone() },
        ]);
        let before = refs(&f);
        let error = apply_protected(&f.git, &f.common, &op, &policy).unwrap_err();
        let diagnostic = error.to_string();
        assert!(diagnostic.contains(PROTECTED) && diagnostic.contains("release-policy"), "{diagnostic}");
        match error {
            OplogError::Protected { refname, current, new: refused, by } => {
                assert_eq!(refname, PROTECTED, "{case}");
                assert_eq!(current, Some(child.clone()), "{case}");
                assert_eq!(refused, *new, "{case}");
                assert_eq!(by, "release-policy", "{case}");
            }
            other => panic!("{case}: expected Protected, got {other:?}"),
        }
        assert!(!oplog_path(&f.common).exists(), "{case}: wrote an intent");
        assert!(read_op_log(&f.common).unwrap().records.is_empty());
        assert_eq!(refs(&f), before, "{case}");
        assert_eq!(f.git.read_ref(PROTECTED).unwrap(), Some(child), "{case}");
    }
}

#[test]
fn undo_and_restore_cannot_rewind_a_protected_ref() {
    let f = Fixture::new();
    let root = commit(&f.git, "root", &[]);
    let child = commit(&f.git, "child", &[&root]);
    let policy = [ProtectedRef { refname: PROTECTED.into(), by: "release-policy".into() }];
    for (id, old, new) in [("checkpoint", None, &root), ("advance", Some(root.clone()), &child)] {
        let op = test_op(id, "publisher", vec![RefMove { refname: PROTECTED.into(), old, new: new.clone() }]);
        assert!(matches!(apply_protected(&f.git, &f.common, &op, &policy).unwrap(), ApplyOutcome::Done(_)));
    }
    let bytes = fs::read(oplog_path(&f.common)).unwrap();
    for action in ["undo", "restore"] {
        let result = if action == "undo" {
            undo(&f.git, &f.common, &[PROTECTED.into()], &caller("undo-protected"), &policy)
        } else {
            restore(&f.git, &f.common, "checkpoint", &[PROTECTED.into()], &caller("restore-protected"), &policy)
        };
        match result.unwrap_err() {
            OplogError::Protected { refname, current, new, by } => {
                assert_eq!(refname, PROTECTED, "{action}");
                assert_eq!(current, Some(child.clone()), "{action}");
                assert_eq!(new, root, "{action}");
                assert_eq!(by, "release-policy", "{action}");
            }
            other => panic!("{action}: expected Protected, got {other:?}"),
        }
        assert_eq!(fs::read(oplog_path(&f.common)).unwrap(), bytes, "{action}");
        assert_eq!(f.git.read_ref(PROTECTED).unwrap(), Some(child.clone()), "{action}");
    }
}
