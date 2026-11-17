//! status joined: `mortar status` reports joined only when the folder's saved work is the joined request tip's.
//! Work snapshotted after a join but never saved to the request branch is unsaved and not joined, next mortar save
//! (S-144: a snapshot is not a save);
//! a join with nothing new since, and a formatted save then a join, stay joined, next "wait for the zone's tests".
//! Status reads only (S-85): refs, index, record, op log and files are unchanged by it (observe::status_at).
//! ZIT (S-119: the subject decides the group): the mortar binary runs as a child in a throwaway repository, with
//! the snapshot crate called directly for the snapshot-only step. The RIT cases of status stay in
//! tests/operational_rit/status.rs.
#[allow(dead_code)]
#[path = "cli_rit/json.rs"]
mod json;
#[allow(dead_code)]
#[path = "cli_rit/support.rs"]
mod support;
#[allow(dead_code)]
#[path = "operational_rit/observe.rs"]
mod observe;

use cli::{snapshot_settings_for, FormatterConfig};
use oplog::{with_lock, KillPidProbe, LockName};
use observe::status;
use support::{script, Fixture, SNAP};

/// Snapshots the request folder as save and join do (the cli's settings for this config and the target
/// snapshot_target_for gives the folder's record, actor "core/task"), without saving the request branch: under
/// the join lock, recover first (snapshot_formatted's header: the caller runs recover and holds the lock).
fn snapshot_only(f: &Fixture) -> snapshot::SnapshotOutcome {
    let settings = snapshot_settings_for(&f.config);
    let record = folders::make::read_folder_record(&f.git.git_dir(&f.request).unwrap()).unwrap().unwrap();
    let target = cli::save::snapshot_target_for(&record, "core/task").unwrap();
    with_lock(&f.git, &f.common, &LockName::Join("core".into()), "zit-snapshot-only", &KillPidProbe, || {
        oplog::recover(&f.git, &f.common).map_err(|e| format!("{e:?}"))?;
        snapshot::snapshot(&f.local(), &f.request, &target, &settings.excludes, settings.cap_bytes)
            .map_err(|e| format!("{e:?}"))
    })
    .unwrap()
}

#[test]
fn snapshot_only_work_after_a_join_is_not_joined() {
    let f = Fixture::new(); f.save_topic();
    f.report(&["--json", "join"], "join", 0, "joined");
    let tip = f.head();
    f.write("src/file", b"work after the join, snapshotted only\n");
    let outcome = snapshot_only(&f);
    assert!(outcome.written, "the snapshot must record the new work");
    assert_eq!(f.git.read_ref(SNAP).unwrap(), Some(outcome.commit.clone()));
    assert_eq!(f.blob(&outcome.commit, "src/file"), b"work after the join, snapshotted only\n");
    assert_eq!(f.head(), tip, "the request branch must not move: the new work was never saved to it");
    // status_at compares observe::before around the call: status reads only.
    // A snapshot is not a save: the folder's tree is the snapshot's, not the tip's, so it reads unsaved (S-144).
    let report = status(&f, "unsaved");
    assert!(!report.flag("saved")); assert!(!report.flag("joined"));
    assert_eq!(report.text("next"), "mortar save \"<sentence>\"");
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

#[test]
fn a_formatted_save_then_join_stays_joined() {
    let f = Fixture::new();
    let formatter = f.root.join("format.sh");
    script(&formatter, b"#!/bin/sh\nfor p in \"$@\"; do printf 'FORMATTED\\n' > \"$p\"; done\n");
    let mut config = f.config.clone();
    config.formatter = Some(FormatterConfig { command: vec![formatter.to_str().unwrap().into()], timeout_seconds: 30 });
    f.configure(&config);
    f.write("src/file", b"unformatted work\n");
    f.report(&["--json", "save", "Save formatted work"], "save", 0, "saved");
    assert_eq!(f.blob(&f.head(), "src/file"), b"FORMATTED\n", "the formatter must have run on the save");
    assert_eq!(std::fs::read(f.request.join("src/file")).unwrap(), b"FORMATTED\n");
    f.report(&["--json", "join"], "join", 0, "joined");
    let report = status(&f, "joined");
    assert!(report.flag("saved")); assert!(report.flag("joined"));
    assert_eq!(report.text("next"), "wait for the zone's tests");
}
