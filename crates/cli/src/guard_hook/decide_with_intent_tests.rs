//! decide_with_intent with land::guard_decide itself (pure: no repository). Guarded refs are refs/heads/main and
//! refs/heads/integration on remote origin, whose push URL is ORIGIN_URL; expected verdicts and call counts are
//! literals from the contracts of decide_with_intent and guard_decide.

use std::cell::Cell;
use std::convert::Infallible;

use gitio::Oid;
use land::{guard_decide, GuardHook, GuardVerdict, LandRefs};
use oplog::RefMove;

use super::decide_with_intent::decide_with_intent;

const A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const ORIGIN_URL: &str = "git@example.com:team/app.git";
const FORK_URL: &str = "git@example.com:me/app.git";

fn refs() -> LandRefs {
    LandRefs { origin_branch: "main".into(), integration_branch: "integration".into(), remote: "origin".into() }
}

fn tx(state: &str) -> GuardHook {
    GuardHook::ReferenceTransaction { state: state.into() }
}

fn push(remote: &str, url: &str, push_urls: Option<&[&str]>) -> GuardHook {
    let push_urls = push_urls.map(|urls| urls.iter().map(|u| u.to_string()).collect());
    GuardHook::PrePush { remote: remote.into(), url: url.into(), push_urls }
}

/// A transaction moving refs/heads/integration from A to B.
fn guarded() -> String {
    format!("{A} {B} refs/heads/integration\n")
}

/// A push moving origin's refs/heads/main from B to A.
fn guarded_push() -> String {
    format!("refs/heads/main {A} refs/heads/main {B}\n")
}

fn integration_move() -> RefMove {
    RefMove { refname: "refs/heads/integration".into(), old: Some(Oid::parse(A).unwrap()), new: Oid::parse(B).unwrap() }
}

/// decide_with_intent's verdict and how many times it called the reader, which gives `moves`.
fn decide(hook: &GuardHook, input: &str, moves: Option<Vec<RefMove>>) -> (GuardVerdict, u32) {
    let calls = Cell::new(0);
    let verdict = decide_with_intent(hook, input, &refs(), || {
        calls.set(calls.get() + 1);
        Ok::<_, Infallible>(moves)
    });
    match verdict {
        Ok(verdict) => (verdict, calls.get()),
        Err(never) => match never {},
    }
}

fn is_refuse(verdict: &GuardVerdict) -> bool {
    matches!(verdict, GuardVerdict::Refuse { .. })
}

#[test]
fn committed_and_aborted_never_read_the_intent() {
    for state in ["committed", "aborted"] {
        assert_eq!(decide(&tx(state), &guarded(), None), (GuardVerdict::Allow, 0), "{state}");
    }
}

#[test]
fn prepared_guarded_with_its_move_in_the_intent_reads_once_and_allows() {
    assert_eq!(decide(&tx("prepared"), &guarded(), Some(vec![integration_move()])), (GuardVerdict::Allow, 1));
}

#[test]
fn prepared_guarded_without_an_intent_reads_once_and_refuses() {
    let (verdict, calls) = decide(&tx("prepared"), &guarded(), None);
    assert_eq!(calls, 1);
    let GuardVerdict::Refuse { why, .. } = verdict else { panic!("expected a refusal: {verdict:?}") };
    assert_eq!(why, "only Mortar moves refs/heads/integration");
}

#[test]
fn prepared_unguarded_reads_once_and_allows() {
    let input = format!("{A} {B} refs/heads/feature\n");
    assert_eq!(decide(&tx("prepared"), &input, None), (GuardVerdict::Allow, 1));
}

#[test]
fn prepared_gives_back_the_readers_error() {
    let verdict = decide_with_intent(&tx("prepared"), &guarded(), &refs(), || Err("log unreadable"));
    assert_eq!(verdict, Err("log unreadable"));
}

#[test]
fn push_to_the_guarded_remote_reads_once() {
    let (verdict, calls) = decide(&push("origin", ORIGIN_URL, Some(&[ORIGIN_URL])), &guarded_push(), None);
    assert_eq!(calls, 1);
    assert!(is_refuse(&verdict), "{verdict:?}");
}

#[test]
fn push_under_another_name_with_unread_urls_reads_once_and_refuses() {
    let (verdict, calls) = decide(&push("fork", FORK_URL, None), &guarded_push(), None);
    assert_eq!(calls, 1);
    assert!(is_refuse(&verdict), "{verdict:?}");
}

#[test]
fn push_elsewhere_never_reads_the_intent() {
    assert_eq!(decide(&push("fork", FORK_URL, Some(&[ORIGIN_URL])), &guarded_push(), None), (GuardVerdict::Allow, 0));
}

/// The rule the probe rests on: a hook that lets the unreadable line through lets every input through.
#[test]
fn a_hook_that_allows_the_unreadable_line_allows_every_input() {
    let hooks = [
        tx("committed"),
        tx("aborted"),
        tx("preparing"),
        tx("prepared"),
        push("origin", ORIGIN_URL, Some(&[ORIGIN_URL])),
        push("origin", ORIGIN_URL, None),
        push("fork", FORK_URL, Some(&[ORIGIN_URL])),
        push("fork", ORIGIN_URL, Some(&[ORIGIN_URL])),
        push("fork", FORK_URL, None),
    ];
    let inputs = [
        guarded(),
        format!("{A} {B} refs/heads/feature\n"),
        guarded_push(),
        format!("refs/heads/feature {A} refs/heads/feature {B}\n"),
        "unreadable".to_string(),
        "garbage line\n".to_string(),
        format!("{A} {B} refs/heads/feature\n{A} {B} refs/heads/main\n"),
        format!("refs/heads/x {A} refs/heads/x {B}\nrefs/heads/main {A} refs/heads/main {B}\n"),
    ];
    let mut allowing = 0;
    for hook in &hooks {
        if guard_decide(hook, "unreadable", &refs(), None) != GuardVerdict::Allow {
            continue;
        }
        allowing += 1;
        for input in &inputs {
            assert_eq!(guard_decide(hook, input, &refs(), None), GuardVerdict::Allow, "{hook:?} {input:?}");
        }
    }
    assert_eq!(allowing, 4, "committed, aborted, preparing and the push to fork");
}

#[test]
fn the_unreadable_line_is_refused_where_the_input_is_read() {
    for hook in [tx("prepared"), push("origin", ORIGIN_URL, Some(&[ORIGIN_URL])), push("fork", FORK_URL, None)] {
        assert!(is_refuse(&guard_decide(&hook, "unreadable", &refs(), None)), "{hook:?}");
    }
}
