//! status joined: `mr status` reports joined only when the folder's saved work is the joined request tip's.
//! Work snapshotted after a join (`mr snapshot`) but never saved to the request branch is unsaved and not joined,
//! next mr save (a snapshot is not a save);
//! a join with nothing new since, and a formatted save (the formatter set by `mr init --replace`) then a join, stay
//! joined, next "wait for the zone's tests".
//! Status reads only: refs, index, record, op log and files are unchanged by it (observe::status_at).
//! Validation test: every step drives the mortar binary as a child in a throwaway repository, reaching the
//! snapshot, join, folders and oplog crates below cli through it; the cli lane owns it. More status
//! RITs are in tests/operational_rit/status.rs.

use crate::observe::status;
use crate::support::{script, Fixture, SNAP};
use gitio::Oid;

/// `mr snapshot` in the request folder as the harness runs it (actor core/task): the folder's work goes to its
/// snapshot ref without a save to the request branch. Returns its report's commit and whether it was written.
fn snapshot_only(f: &Fixture) -> (Oid, bool) {
    let report = f.report(&["--json", "--actor", "core/task", "snapshot"], "snapshot", 0, "snapshotted");
    (Oid::parse(report.text("commit")).unwrap(), report.flag("written"))
}

#[test]
fn snapshot_only_work_after_a_join_is_not_joined() {
    let f = Fixture::new(); f.save_topic();
    f.report(&["--json", "join"], "join", 0, "joined");
    let tip = f.head();
    f.write("src/file", b"work after the join, snapshotted only\n");
    let (commit, written) = snapshot_only(&f);
    assert!(written, "the snapshot must record the new work");
    assert_eq!(f.git.read_ref(SNAP).unwrap(), Some(commit.clone()));
    assert_eq!(f.blob(&commit, "src/file"), b"work after the join, snapshotted only\n");
    assert_eq!(f.head(), tip, "the request branch must not move: the new work was never saved to it");
    // status_at compares observe::before around the call: status reads only.
    // A snapshot is not a save: the folder's tree is the snapshot's, not the tip's, so it reads unsaved.
    let report = status(&f, "unsaved");
    assert!(!report.flag("saved")); assert!(!report.flag("joined"));
    assert_eq!(report.text("next"), "mr save \"<sentence>\"");
    assert_eq!(f.head(), tip);
}

#[test]
fn joined_with_nothing_new_stays_joined() {
    let f = Fixture::new(); f.save_topic();
    f.report(&["--json", "join"], "join", 0, "joined");
    let report = status(&f, "joined");
    assert!(report.flag("saved")); assert!(report.flag("joined"));
    assert_eq!(report.text("next"), "wait for the zone's tests");
}

/// `mr init --replace` with the fixture's config written out as flags (Fixture::new's values), plus the formatter:
/// formatter is the one key that changes.
fn set_formatter(f: &Fixture, formatter: &str) {
    let report = f.report(&["--json", "--actor", "harness", "init", "--origin", "main", "--integration", "integration",
        "--remote", "origin", "--zone", "core=src,sub", "--root-file", "root.txt", "--build-dir", "d/",
        "--cap-bytes", &snapshot::DEFAULT_SIZE_CAP.to_string(), "--floor-bytes", "0", "--formatter", formatter, "--formatter-timeout", "30", "--replace"],
        "init", 0, "replaced");
    assert!(report.lines().ends_with("; changed: formatter"), "{}", report.lines());
}

#[test]
fn a_formatted_save_then_join_stays_joined() {
    let f = Fixture::new();
    let formatter = f.root.join("format.sh");
    script(&formatter, b"#!/bin/sh\nfor p in \"$@\"; do printf 'FORMATTED\\n' > \"$p\"; done\n");
    set_formatter(&f, formatter.to_str().unwrap());
    f.write("src/file", b"unformatted work\n");
    f.report(&["--json", "save", "Save formatted work"], "save", 0, "saved");
    assert_eq!(f.blob(&f.head(), "src/file"), b"FORMATTED\n", "the formatter must have run on the save");
    assert_eq!(std::fs::read(f.request.join("src/file")).unwrap(), b"FORMATTED\n");
    f.report(&["--json", "join"], "join", 0, "joined");
    let report = status(&f, "joined");
    assert!(report.flag("saved")); assert!(report.flag("joined"));
    assert_eq!(report.text("next"), "wait for the zone's tests");
}
