//! #1019, S-52(1)/S-53/S-54: sweep_refs joins real branch containment/worktree discovery, oplog and maintenance.
use super::maintenance_support::*;
use crate::support::run;
use folders::FolderError;
use oplog::ProtectedRef;
use std::fs;

#[test]
fn sweep_deletes_closed_snapshot_joined_leftover_and_extra_but_keeps_open_unjoined_and_checked_out() {
    let m = Removal::new(); m.snapshot();
    let pending = child(&m.f, "unjoined work");
    for (name, id) in [
        ("refs/heads/req/red/left", &m.f.head), ("refs/mortar/snap/red/left", &m.f.head),
        ("refs/heads/req/red/pending", &pending), ("refs/mortar/snap/red/pending", &pending),
        ("refs/mortar/snap/red/closed", &m.f.head), ("refs/checkpoints/closed", &m.f.head),
    ] { run(&m.f.git, &["update-ref", name, id.as_str()]); }
    let mut expected = refs(&m.f.git);
    let deleted = ["refs/mortar/snap/red/closed", "refs/heads/req/red/left", "refs/mortar/snap/red/left", "refs/checkpoints/closed"];
    for name in deleted { expected.remove(name); }
    let start = records(&m.f).len();
    let (git, _, maintenance) = instrument(&m.f, "observe", "", &m.f.head);
    assert_eq!(sweep(&m.f, &git, &[], &["refs/checkpoints/closed".into(), "refs/checkpoints/absent".into(), "refs/checkpoints/closed".into()], &[]).unwrap(), deleted.map(str::to_string));
    assert_eq!(refs(&m.f.git), expected);
    assert_eq!(m.f.git.read_ref(BRANCH).unwrap(), Some(m.f.head.clone()), "joined but checked-out request is kept");
    assert_eq!(m.f.git.read_ref(SNAP).unwrap(), Some(m.f.head.clone()));
    assert_eq!(m.f.git.read_ref("refs/heads/req/red/pending").unwrap(), Some(pending));
    assert_eq!(fs::read(m.folder.join("kit/zone/file")).unwrap(), b"zone\n");
    let ops = records(&m.f); assert_delete_pair(&ops[start..], "folder-sweep-refs", &deleted, &m.f.head);
    assert_eq!(fs::read(&maintenance).unwrap(), b"maintenance\n");
    let before_log = log(&m.f);
    assert!(sweep(&m.f, &git, &[], &["refs/checkpoints/absent".into()], &[]).unwrap().is_empty());
    assert_eq!(log(&m.f), before_log, "empty selection logs no op");
    assert_eq!(fs::read(maintenance).unwrap(), b"maintenance\nmaintenance\n", "maintenance still runs with no refs to sweep");
    assert_eq!(refs(&m.f.git), expected);
}

#[test]
fn forbidden_namespace_and_caller_protected_extras_refuse_before_any_op_or_deletion_even_when_absent() {
    for name in ["refs/heads/keep", "refs/mortar/keep", "refs/remotes/origin/keep", "refs/checkpoints/protected"] {
        for present in [false, true] {
            let m = Removal::new();
            for candidate in ["refs/mortar/snap/red/closed", "refs/checkpoints/allowed"] {
                run(&m.f.git, &["update-ref", candidate, m.f.head.as_str()]);
            }
            if present { run(&m.f.git, &["update-ref", name, m.f.head.as_str()]); }
            let protected = [ProtectedRef { refname: "refs/checkpoints/protected".into(), by: "fixture-owner".into() }];
            let before_refs = refs(&m.f.git); let before_log = log(&m.f);
            let (git, _, maintenance) = instrument(&m.f, "observe", "", &m.f.head);
            let error = sweep(&m.f, &git, &[], &["refs/checkpoints/allowed".into(), name.into()], &protected).unwrap_err();
            match error {
                FolderError::ExtraRefRefused { refname, reason } => {
                    assert_eq!(refname, name);
                    if name == "refs/checkpoints/protected" { assert_eq!(reason, "is protected by fixture-owner"); }
                    else { assert!(reason.starts_with("is under refs/"), "{reason}"); }
                }
                other => panic!("expected ExtraRefRefused: {other:?}"),
            }
            assert_eq!(refs(&m.f.git), before_refs, "present={present} {name}");
            assert_eq!(log(&m.f), before_log);
            assert!(!maintenance.exists(), "refusal must precede maintenance too");
        }
    }
}

#[test]
fn sweep_cas_race_names_changed_ref_and_deletes_none_of_the_other_selected_refs() {
    let m = Removal::new();
    let closed = "refs/mortar/snap/red/closed";
    let extra = "refs/checkpoints/closed";
    for name in [closed, extra] { run(&m.f.git, &["update-ref", name, m.f.head.as_str()]); }
    let next = child(&m.f, "concurrent extra ref");
    let mut expected = refs(&m.f.git); expected.insert(extra.into(), next.to_string());
    let (git, marker, maintenance) = instrument(&m.f, "sweep-race", extra, &next);
    let error = sweep(&m.f, &git, &[], &[extra.into()], &[]).unwrap_err();
    assert!(matches!(error, FolderError::RefMoved { refname, at } if refname == extra && at == Some(next)));
    assert_eq!(fs::read(marker).unwrap(), b"raced\n");
    assert_eq!(refs(&m.f.git), expected, "transaction must not delete the closed snapshot either");
    assert!(!maintenance.exists());
    assert!(matches!(&records(&m.f).last().unwrap().result, Some(oplog::OpResult::Stale { refname, actual: Some(_) }) if refname == extra));
}
