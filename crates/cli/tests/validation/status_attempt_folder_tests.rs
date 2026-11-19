//! status in an attempt folder: in attempt folder k, `mr status` exits 0 and reports attempt k's
//! own progress, its branch try/<zone>/<request>/<k> and its trysnap snapshot, never the request's. Its next is
//! `mr save "<sentence>"` while the attempt is unsaved, "make your change, then mr save" while a fresh
//! attempt has no work yet, otherwise "wait for the harness to keep or drop the attempts", never join, even
//! when the zone has moved. `need` stays refused in an attempt folder.
//! Status reads only: refs, index, record, op log and files are unchanged by it (observe::status_at).
//! Validation test: the mortar binary runs as a child in a throwaway repository, with the attempts made by the
//! folders crate directly; the cli lane owns it.

use crate::json::check;
use crate::observe::{before, status_at};
use crate::support::{raw, setup, write, Fixture, ZONE};
use std::path::{Path, PathBuf};

const WAIT: &str = "wait for the harness to keep or drop the attempts";
const TRY_2: &str = "refs/heads/try/core/task/2";

/// Makes two attempts of core/task under the fixture's root; returns attempt 1's and attempt 2's folders.
fn attempts(f: &Fixture) -> (PathBuf, PathBuf) {
    let dir = f.root.join("attempts");
    oplog::recover(&f.git, &f.common).unwrap();
    folders::make_attempts(&setup(&f.git, &dir), "core", "task", &["src".into()], 2).unwrap();
    (dir.join("try-1"), dir.join("try-2"))
}

/// Writes and saves work in the attempt folder; the request branch must not move.
fn save_in(f: &Fixture, attempt: &Path) {
    let tip = f.head();
    write(attempt, "src/file", b"attempt work\n");
    check(f.command(attempt, &["--json", "save", "Attempt work"]).output().unwrap(), "save", 0, "saved");
    assert_eq!(f.head(), tip, "a save in an attempt must not move the request branch");
}

/// A fresh attempt's try branch is at its base and its tree is the tip's: saved with no work yet.
#[test]
fn fresh_attempt_reports_saved_and_next_make_your_change() {
    let f = Fixture::new(); f.save_topic();
    let (one, _) = attempts(&f);
    let report = status_at(&f, &one, "saved");
    assert!(report.flag("saved"));
    assert_eq!(report.text("next"), "make your change, then mr save");
    assert_eq!(report.number("attempt"), 1);
    assert_eq!(report.text("branch"), "try/core/task/1");
    assert!(report.lines().contains("try/core/task/1"));
}

#[test]
fn saved_attempt_waits_for_keep_or_drop() {
    let f = Fixture::new();
    let (one, two) = attempts(&f);
    let other = f.git.read_ref(TRY_2).unwrap();
    save_in(&f, &one);
    assert_eq!(f.git.read_ref(TRY_2).unwrap(), other, "a save in attempt 1 must not move attempt 2's branch");
    let report = status_at(&f, &one, "saved");
    assert!(report.flag("saved"));
    assert_eq!(report.text("next"), WAIT);
    assert!(!report.text("next").contains("join"));
    // Attempt 2 is still fresh: saved with no work yet.
    let second = status_at(&f, &two, "saved");
    assert_eq!(second.number("attempt"), 2);
    assert_eq!(second.text("branch"), "try/core/task/2");
}

#[test]
fn zone_moved_still_never_join() {
    let f = Fixture::new();
    let (one, _) = attempts(&f);
    save_in(&f, &one);
    f.advance_zone(false);
    let report = status_at(&f, &one, "saved");
    assert!(report.flag("zone_moved"));
    assert_eq!(report.text("next"), WAIT);
    assert!(!report.text("next").contains("join"));
}

#[test]
fn need_in_an_attempt_is_still_refused() {
    let f = Fixture::new();
    let (one, _) = attempts(&f);
    let old = before(&f, &one);
    check(f.command(&one, &["--json", "need", "src"]).output().unwrap(), "need", 1, "refused");
    assert_eq!(before(&f, &one), old);
}

#[test]
fn zone_at_the_attempt_tip_is_not_moved_for_the_attempt() {
    let f = Fixture::new(); f.save_topic();
    let (one, _) = attempts(&f);
    save_in(&f, &one);
    let tip = f.git.read_ref("refs/heads/try/core/task/1").unwrap().unwrap();
    raw(&f.git, &["update-ref", ZONE, tip.as_str()]);
    // The zone tip is attempt 1's saved tip: read from try/core/task/1 the zone has nothing the attempt lacks and
    // the attempt's work is in the zone. A read of req/core/task instead gives zone_moved true and joined false.
    let report = status_at(&f, &one, "joined");
    assert!(!report.flag("zone_moved"));
    assert!(report.flag("joined"));
    assert_eq!(report.text("next"), WAIT);
}
