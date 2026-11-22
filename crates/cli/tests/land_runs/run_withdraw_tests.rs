//! `mortar withdraw <zone>`: a zone's squash withdrawn before it lands, its squash ref deleted and its commit and
//! change ID answered; a repeat has nothing to withdraw; a squash already in origin and the squash that is the
//! landing in flight are refused with the ref left; no zone, a second word and an unknown zone are usage errors; a
//! squash after a withdraw draws a new change ID.
//! Drives the mortar binary against real git and the crates below cli; the cli lane owns it. The squash and the
//! landing are made through the binary too (mr squash, mr land).

use crate::json::{check, Report};
use crate::support::{raw, Fixture, ZONE};
use gitio::Oid;

const SQUASH_CORE: &str = "refs/mortar/squash/core";

/// The main repository with HEAD detached, integration at the base and zone/core one commit ahead of main.
fn setup() -> Fixture {
    let f = Fixture::plain();
    raw(&f.git, &["checkout", "-q", "--detach"]);
    raw(&f.git, &["branch", "integration", f.base.as_str()]);
    let zone = f.commit(&f.base, &[("src/file", b"zone work\n")], "zone work\n\nMortar-Request: r1\n");
    raw(&f.git, &["update-ref", ZONE, zone.as_str()]);
    f
}

/// `mortar --json --actor land-runs <words>` in the main repository, checked for verb words[0], exit and outcome.
fn run(f: &Fixture, words: &[&str], exit: i32, outcome: &str) -> Report {
    let mut command = f.command(&f.git.repo, &["--json", "--actor", "land-runs"]);
    check(command.args(words).output().unwrap(), words[0], exit, outcome)
}

fn squash(f: &Fixture) -> Report { run(f, &["squash", "core"], 0, "squashed") }

fn refs(f: &Fixture) -> String {
    String::from_utf8(raw(&f.git, &["for-each-ref", "--format=%(refname) %(objectname)"])).unwrap()
}

fn squash_ref(f: &Fixture) -> Option<Oid> { f.git.read_ref(SQUASH_CORE).unwrap() }

#[test]
fn withdraw_after_a_squash_deletes_the_squash_ref_and_names_its_commit_and_change_id() {
    let f = setup();
    let made = squash(&f);
    let before = refs(&f);
    let r = run(&f, &["withdraw", "core"], 0, "withdrawn");
    assert_eq!(r.text("withdrawn"), made.text("commit"));
    assert_eq!(r.text("change_id"), made.text("change_id"));
    assert_eq!(r.text("zone"), "core");
    assert_eq!(squash_ref(&f), None);
    let squash_line = format!("{SQUASH_CORE} {}\n", made.text("commit"));
    assert!(before.contains(&squash_line));
    assert_eq!(refs(&f), before.replace(&squash_line, ""));
}

#[test]
fn withdraw_repeated_has_nothing_to_withdraw() {
    let f = setup();
    squash(&f);
    run(&f, &["withdraw", "core"], 0, "withdrawn");
    let before = refs(&f);
    let r = run(&f, &["withdraw", "core"], 0, "nothing-to-withdraw");
    assert!(r.is_null("withdrawn"));
    assert_eq!(refs(&f), before);
}

#[test]
fn withdraw_of_a_squash_already_in_origin_is_refused_with_squash_next() {
    let f = setup();
    let made = squash(&f);
    let commit = Oid::parse(made.text("commit")).unwrap();
    let past = f.commit(&commit, &[("src/past", b"past the squash\n")], "past the squash\n");
    raw(&f.git, &["update-ref", "refs/heads/main", past.as_str()]);
    let before = refs(&f);
    let r = run(&f, &["withdraw", "core"], 1, "refused");
    assert!(r.text("next").contains("mr squash core"), "{}", r.text("next"));
    assert_eq!(squash_ref(&f), Some(commit));
    assert_eq!(refs(&f), before);
}

#[test]
fn withdraw_of_the_landing_in_flight_is_refused_with_back_out_then_withdraw_next() {
    let f = setup();
    let made = squash(&f);
    run(&f, &["land", made.text("commit")], 0, "landed");
    let before = refs(&f);
    let r = run(&f, &["withdraw", "core"], 1, "refused");
    let next = r.text("next");
    assert!(next.contains("mr back-out"), "{next}");
    assert!(next.contains("mr withdraw core"), "{next}");
    assert_eq!(squash_ref(&f).unwrap().as_str(), made.text("commit"));
    assert_eq!(refs(&f), before);
}

#[test]
fn withdraw_usage_errors_name_withdraws_usage_and_leave_the_squash_ref() {
    let f = setup();
    squash(&f);
    let before = refs(&f);
    for words in [&["withdraw"][..], &["withdraw", "core", "extra"], &["withdraw", "nope"]] {
        let r = run(&f, words, 4, "usage");
        assert!(r.text("next").starts_with("mr withdraw <zone>"), "{words:?}: {}", r.text("next"));
        assert_eq!(refs(&f), before, "{words:?}");
    }
}

#[test]
fn a_squash_after_a_withdraw_draws_a_new_change_id() {
    let f = setup();
    let first = squash(&f);
    run(&f, &["withdraw", "core"], 0, "withdrawn");
    let second = squash(&f);
    assert_ne!(second.text("change_id"), first.text("change_id"));
    assert_eq!(squash_ref(&f).unwrap().as_str(), second.text("commit"));
}
