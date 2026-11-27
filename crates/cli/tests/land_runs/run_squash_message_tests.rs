//! `mortar squash <zone> --message <text>`: the squash commit's message begins with the caller's text, then the
//! request lines, then Mortar's trailers, and the squash reads back by its change ID; a message squash after a plain
//! one keeps the change ID with a new commit, and the same message again is already-squashed; a --message with no
//! value, an empty or blank value, or given twice is a usage error naming --message, with no squash ref written.
//! Drives the mortar binary against real git and the crates below cli; the cli lane owns it.

use crate::json::{check, Report};
use crate::support::{raw, Fixture, ZONE};
use gitio::Oid;

const SQUASH_CORE: &str = "refs/mortar/squash/core";
const TEXT: &str = "Land the core change\n\nIt carries request r1's work.";

/// The main repository with HEAD detached, integration at the base and zone/core one commit of request r1 ahead of
/// main; the zone tip.
fn setup() -> (Fixture, Oid) {
    let f = Fixture::plain();
    raw(&f.git, &["checkout", "-q", "--detach"]);
    raw(&f.git, &["branch", "integration", f.base.as_str()]);
    let zone = f.commit(&f.base, &[("src/file", b"zone work\n")], "zone work\n\nMortar-Request: r1\n");
    raw(&f.git, &["update-ref", ZONE, zone.as_str()]);
    (f, zone)
}

/// `mortar --json --actor land-runs <words>` in the main repository, checked for verb words[0], exit and outcome.
fn run(f: &Fixture, words: &[&str], exit: i32, outcome: &str) -> Report {
    let mut command = f.command(&f.git.repo, &["--json", "--actor", "land-runs"]);
    check(command.args(words).output().unwrap(), words[0], exit, outcome)
}

fn message(f: &Fixture, commit: &str) -> String {
    String::from_utf8(raw(&f.git, &["log", "-1", "--format=%B", commit])).unwrap()
}

/// Where `needle` first occurs in `haystack`; the test fails when it does not.
fn at(haystack: &str, needle: &str) -> usize {
    haystack.find(needle).unwrap_or_else(|| panic!("no {needle:?} in {haystack:?}"))
}

#[test]
fn a_message_squash_begins_with_the_text_then_the_requests_then_the_trailers_and_reads_back() {
    let (f, zone) = setup();
    let r = run(&f, &["squash", "core", "--message", TEXT], 0, "squashed");
    assert_eq!(r.text("zone"), "core");
    assert_eq!(r.text("source"), zone.as_str());
    assert_eq!(r.strings("requests"), &["r1"]);
    let (commit, change_id) = (r.text("commit"), r.text("change_id"));
    assert_eq!(f.git.read_ref(SQUASH_CORE).unwrap().unwrap().as_str(), commit);
    let body = message(&f, commit);
    assert!(body.starts_with(&format!("{TEXT}\n")), "{body:?}");
    assert!(!body.contains("core: 1 requests"), "{body:?}");
    let request = at(&body, "\n- r1: zone work\n");
    assert!(request > TEXT.len(), "{body:?}");
    for trailer in [format!("Mortar-Change-Id: {change_id}"), "Mortar-Zone: core".into(), format!("Mortar-Squash-Of: {zone}")] {
        assert!(at(&body, &trailer) > request, "{trailer}: {body:?}");
    }
    let read = run(&f, &["would-merge", change_id], 0, "merges");
    assert_eq!(read.text("commit"), commit);
    assert_eq!(read.text("change_id"), change_id);
}

#[test]
fn a_message_after_a_plain_squash_keeps_the_change_id_and_the_same_message_again_is_already_squashed() {
    let (f, _) = setup();
    let plain = run(&f, &["squash", "core"], 0, "squashed");
    let worded = run(&f, &["squash", "core", "--message", TEXT], 0, "squashed");
    assert_eq!(worded.text("change_id"), plain.text("change_id"));
    assert_ne!(worded.text("commit"), plain.text("commit"));
    assert_eq!(f.git.read_ref(SQUASH_CORE).unwrap().unwrap().as_str(), worded.text("commit"));
    let joined = format!("--message={TEXT}");
    let again = run(&f, &["squash", "core", &joined], 0, "already-squashed");
    assert_eq!(again.text("commit"), worded.text("commit"));
    assert_eq!(again.text("change_id"), worded.text("change_id"));
    assert_eq!(f.git.read_ref(SQUASH_CORE).unwrap().unwrap().as_str(), worded.text("commit"));
}

#[test]
fn a_missing_empty_blank_or_repeated_message_is_usage_naming_message_and_writes_no_squash_ref() {
    let (f, _) = setup();
    for words in [
        &["squash", "core", "--message"][..],
        &["squash", "core", "--message", ""],
        &["squash", "core", "--message="],
        &["squash", "core", "--message", "  \n "],
        &["squash", "core", "--message", "one", "--message", "two"],
    ] {
        let r = run(&f, words, 2, "usage");
        let reason = &r.strings("lines")[0];
        assert!(reason.starts_with("--message "), "{words:?}: {reason}");
        assert_eq!(f.git.read_ref(SQUASH_CORE).unwrap(), None, "{words:?}");
    }
}
