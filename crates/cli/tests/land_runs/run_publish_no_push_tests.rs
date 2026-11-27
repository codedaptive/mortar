//! `mr publish --no-push` (S-217, S-220): a landing put on the local origin branch only, the remote never pushed,
//! read or fetched, `pushed` false (a plain publish's `pushed` true is run_publish_tests'); with no landing, nothing
//! to publish; a local origin moved after the landing is origin-moved onto it, saying land again and publish with
//! --no-push; while this publish's push intent is open (a push-on run stopped between its steps) push off is
//! refused and a plain publish finishes it; with that intent closed done and the remote already at the landing (a
//! push-on run killed after its push, before moving the local origin) push off moves only the local origin and
//! reports `pushed` true; a doubled flag or an extra word is a usage error naming the word.
//! Drives the mr binary against real git and the crates below cli; the cli lane owns it.
//! Each case's repository is support's Fixture (zone core over src and sub) with a bare `origin` holding main beside
//! it; the squash and the landing are made in setup through land::squash and land::land_batch under the land lock, the
//! push intent opened through oplog's open_remote_intent, as land's publish_zit fixture.rs (open_push_intent) opens
//! it for repeats.rs, and closed done through oplog's close_intent, as publish's own push closes it.

use std::process::{Command, Output, Stdio};

use crate::json::{check, Report};
use crate::support::{raw, Fixture, ZONE};
use gitio::{Git, Oid};
use land::{BatchLandAnswer, LandRefs, Squash, SquashAnswer};
use oplog::{IntentClose, KillPidProbe, LockName, Op, RefMove, RemoteIntentOutcome};

const SQUASH: &str = "refs/mortar/squash/core";
const MAIN: &str = "refs/heads/main";
const INTEGRATION: &str = "refs/heads/integration";
const SETUP_ACTOR: &str = "publish-no-push-setup";
/// The op id of the push intent a case opens by hand; new_op_id never makes this form.
const OPEN_PUSH_OP_ID: &str = "cli-zit-open-push";

fn refs() -> LandRefs {
    LandRefs { origin_branch: "main".into(), integration_branch: "integration".into(), remote: "origin".into() }
}

/// `act` under the land lock after recover, as the cli runs its landing verbs.
fn locked<T>(f: &Fixture, act: impl FnOnce() -> T) -> T {
    oplog::with_lock(&f.git, &f.common, &LockName::Land, SETUP_ACTOR, &KillPidProbe, || {
        oplog::recover(&f.git, &f.common).unwrap();
        Ok::<T, std::io::Error>(act())
    })
    .unwrap()
}

/// The fixture with HEAD detached (main checked out nowhere), integration at the base, zone core one commit ahead
/// and squashed (not landed), and a bare `origin` at `<root>/origin.git` whose main is the base.
fn squashed() -> (Fixture, Squash) {
    let f = Fixture::new();
    raw(&f.git, &["checkout", "-q", "--detach"]);
    raw(&f.git, &["branch", "integration", f.base.as_str()]);
    let zone = f.commit(&f.base, &[("src/file", b"zone work\n")], "zone work\n");
    raw(&f.git, &["update-ref", ZONE, zone.as_str()]);
    let squash = locked(&f, || {
        let SquashAnswer::Made(squash) = land::squash(&f.git, &refs(), "core", SETUP_ACTOR).unwrap() else {
            panic!("setup squash was not made");
        };
        squash
    });
    let bare = f.root.join("origin.git");
    raw(&f.git, &["init", "-q", "--bare", "--initial-branch=main", bare.to_str().unwrap()]);
    raw(&f.git, &["remote", "add", "origin", bare.to_str().unwrap()]);
    raw(&f.git, &["push", "-q", "origin", "refs/heads/main:refs/heads/main"]);
    (f, squash)
}

/// `squashed`, then the squash landed by land::land_batch, asserted to fast-forward integration to it.
fn landed() -> (Fixture, Squash) {
    let (f, squash) = squashed();
    let answer = locked(&f, || land::land_batch(&f.git, &refs(), &squash, SETUP_ACTOR).unwrap());
    assert_eq!(answer, BatchLandAnswer::Landed { integration: squash.commit.clone() });
    (f, squash)
}

/// `mr <args>` run in the main repository, git's config isolated.
fn mr(f: &Fixture, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_mr")).current_dir(&f.git.repo).env_clear().envs(f.git.env.iter().cloned())
        .env("PATH", std::env::var_os("PATH").unwrap()).env("HOME", &f.root).env("TMPDIR", &f.root)
        .env("CODETEAM_SESSION", "publish-no-push-test").stdin(Stdio::null()).args(args).output().unwrap()
}

fn no_push(f: &Fixture, exit: i32, outcome: &str) -> Report {
    check(mr(f, &["publish", "--no-push", "--actor", "a", "--json"]), "publish", exit, outcome)
}

fn plain(f: &Fixture, exit: i32, outcome: &str) -> Report {
    check(mr(f, &["publish", "--actor", "a", "--json"]), "publish", exit, outcome)
}

fn local(f: &Fixture, refname: &str) -> Option<Oid> {
    f.git.read_ref(refname).unwrap()
}

/// The remote's main, read in the bare repository itself.
fn remote_main(f: &Fixture) -> Oid {
    Git { repo: f.root.join("origin.git"), ..f.git.clone() }.read_ref(MAIN).unwrap().unwrap()
}

/// Every remote-tracking ref and every ref publish fetches into, with its commit.
fn fetched_refs(f: &Fixture) -> Vec<u8> {
    raw(&f.git, &["for-each-ref", "--format=%(refname) %(objectname)", "refs/remotes", "refs/mortar/fetched"])
}

/// Opens this publish's push intent (the remote's main from the base to the squash), marked with the remote
/// `origin`, as a push-on publish opens it before its push.
fn open_push_intent(f: &Fixture, squash: &Squash) {
    let op = Op {
        op_id: OPEN_PUSH_OP_ID.into(),
        // land's publish_push_op key form (pub(crate) in land): the key publish looks its push intent up by.
        key: format!("publish-push:origin:{MAIN}:{}:{}", f.base, squash.commit),
        kind: "publish-push".into(),
        actor: SETUP_ACTOR.into(),
        reason: "publish-push opened by the test".into(),
        at: 1_700_000_000,
        moves: vec![RefMove { refname: MAIN.into(), old: Some(f.base.clone()), new: squash.commit.clone() }],
    };
    let outcome = oplog::open_remote_intent(&f.git, &f.common, &op, "origin").unwrap();
    assert!(matches!(outcome, RemoteIntentOutcome::Opened(_)), "{outcome:?}");
}

#[test]
fn no_push_after_a_landing_moves_only_the_local_origin_and_removes_the_squash_ref() {
    let (f, _) = landed();
    let integration = local(&f, INTEGRATION).unwrap();
    let report = no_push(&f, 0, "published");
    assert!(!report.flag("pushed"));
    assert_eq!(report.text("origin"), integration.as_str());
    assert_eq!(local(&f, MAIN), Some(integration));
    assert_eq!(remote_main(&f), f.base);
    assert_eq!(local(&f, SQUASH), None);
}

#[test]
fn no_push_with_no_landing_has_nothing_to_publish() {
    let (f, _) = squashed();
    let report = no_push(&f, 0, "nothing-to-publish");
    assert!(!report.flag("pushed"));
}

#[test]
fn no_push_after_the_local_origin_moved_is_origin_moved_onto_it_without_fetching() {
    let (f, _) = landed();
    let moved = f.commit(&f.base, &[("src/other", b"local origin work\n")], "local origin moved\n");
    raw(&f.git, &["update-ref", MAIN, moved.as_str()]);
    let remote = f.commit(&f.base, &[("src/file", b"remote work\n")], "remote advanced\n");
    raw(&f.git, &["push", "-q", "origin", &format!("{remote}:refs/heads/main")]);
    let fetched = fetched_refs(&f);
    let report = no_push(&f, 3, "origin-moved");
    assert_eq!(report.text("found"), moved.as_str());
    assert_eq!(local(&f, MAIN), Some(moved.clone()));
    assert_eq!(local(&f, INTEGRATION), Some(moved));
    assert_eq!(remote_main(&f), remote);
    assert_eq!(fetched_refs(&f), fetched);
    let next = report.text("next");
    assert!(next.contains("mr land"), "{next}");
    assert!(next.ends_with("mr publish --no-push --actor <name>"), "{next}");
}

#[test]
fn no_push_while_the_push_intent_is_open_is_refused_then_a_plain_publish_finishes() {
    let (f, squash) = landed();
    open_push_intent(&f, &squash);
    let report = no_push(&f, 3, "refused");
    let next = report.text("next");
    assert!(next.starts_with("mr publish"), "{next}");
    assert!(!next.contains("--no-push"), "{next}");
    assert_eq!(local(&f, MAIN), Some(f.base.clone()));
    assert_eq!(local(&f, INTEGRATION), Some(squash.commit.clone()));
    assert_eq!(local(&f, SQUASH), Some(squash.commit.clone()));
    assert_eq!(remote_main(&f), f.base);
    let report = plain(&f, 0, "published");
    assert!(report.flag("pushed"));
    assert_eq!(remote_main(&f), squash.commit);
}

#[test]
fn no_push_after_a_push_on_run_pushed_and_closed_its_intent_done_reports_pushed() {
    let (f, squash) = landed();
    raw(&f.git, &["push", "-q", "origin", &format!("{}:refs/heads/main", squash.commit)]);
    open_push_intent(&f, &squash);
    oplog::close_intent(&f.common, OPEN_PUSH_OP_ID, IntentClose::Done).unwrap();
    assert_eq!(local(&f, MAIN), Some(f.base.clone()));
    let report = no_push(&f, 0, "published");
    assert!(report.flag("pushed"));
    assert_eq!(report.text("origin"), squash.commit.as_str());
    assert_eq!(local(&f, MAIN), Some(squash.commit.clone()));
    assert_eq!(remote_main(&f), squash.commit);
}

#[test]
fn a_doubled_no_push_or_an_extra_word_is_usage_naming_the_word() {
    let (f, _) = landed();
    let rows: [(&[&str], &str); 2] = [
        (&["publish", "--no-push", "--no-push", "--actor", "a", "--json"], "--no-push"),
        (&["publish", "now", "--actor", "a", "--json"], "now"),
    ];
    for (args, word) in rows {
        let report = check(mr(&f, args), "publish", 2, "usage");
        let reason = &report.strings("lines")[0];
        assert!(reason.contains(word), "{args:?}: {reason}");
        let next = report.text("next");
        assert!(next.starts_with("mr publish "), "{args:?}: {next}");
        assert!(next.contains("[--no-push]"), "{args:?}: {next}");
    }
}
