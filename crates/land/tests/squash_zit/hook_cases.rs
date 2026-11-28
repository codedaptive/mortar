//! Squash hooks can refuse or rewrite the message before the repeat check; unreadable rewrites are refused.
//! Nothing and conflicted squashes run no hooks.

use crate::fixture::{made, oid, refs, Fixture, ACTOR, SQUASH_REF, ZONE};
use gitio::HookRefusal;
use land::{squash_with_hooks, HookedSquashError, SquashAnswer};

#[test]
fn a_refusing_hook_stops_the_squash_and_moves_nothing() {
    for (hook, script, code) in [
        ("pre-commit", "#!/bin/sh\nprintf 'hook output\n'\nprintf 'hook error\n' >&2\nexit 7\n", 7),
        ("commit-msg", "#!/bin/sh\nprintf 'hook output\n'\nprintf 'hook error\n' >&2\nexit 9\n", 9),
    ] {
        let fx = Fixture::new();
        let z = fx.commit_on(&fx.base, &[("z", "zone\n")], "add z", Some("r1"));
        fx.set_branch("zone/land", &z);
        fx.install_hook(hook, script);
        let log = fx.oplog();

        let Err(HookedSquashError::Refused(refusal)) =
            squash_with_hooks(&fx.land_git(), &refs(), ZONE, ACTOR, Some("Fix the guard")) else {
                panic!("{hook} must refuse the squash");
            };
        assert_eq!(refusal, HookRefusal {
            hook, code: Some(code), stdout: b"hook output\n".to_vec(), stderr: b"hook error\n".to_vec(),
        });
        assert_eq!(fx.ref_of(SQUASH_REF), None);
        assert_eq!(fx.oplog(), log);
    }
}

#[test]
fn a_commit_msg_hook_that_rewrites_the_message_is_what_is_committed_and_a_repeat_is_same() {
    let fx = Fixture::new();
    let z = fx.commit_on(&fx.base, &[("z", "zone\n")], "add z", Some("r1"));
    fx.set_branch("zone/land", &z);
    fx.install_hook("commit-msg", "#!/bin/sh\n{ printf 'Hooked: '; cat \"$1\"; } > \"$1.tmp\"\nmv \"$1.tmp\" \"$1\"\n");

    let first = made(squash_with_hooks(&fx.land_git(), &refs(), ZONE, ACTOR, Some("Fix the guard")).unwrap());
    let expected = format!(
        "Hooked: Fix the guard\n\n- r1: add z\n\nMortar-Change-Id: {}\nMortar-Zone: land\nMortar-Squash-Of: {z}\n",
        first.change_id.as_str()
    );
    assert_eq!(fx.message(&first.commit.to_string()), expected);
    let log = fx.oplog();
    assert!(log.is_some());

    let again = squash_with_hooks(&fx.land_git(), &refs(), ZONE, ACTOR, Some("Fix the guard")).unwrap();
    assert_eq!(again, SquashAnswer::Same(first.clone()));
    assert_eq!(fx.ref_of(SQUASH_REF), Some(first.commit.to_string()));
    assert_eq!(fx.oplog(), log);
}

#[test]
fn a_refusing_pre_commit_refuses_a_repeat_that_would_be_same() {
    let fx = Fixture::new();
    let z = fx.commit_on(&fx.base, &[("z", "zone\n")], "add z", Some("r1"));
    fx.set_branch("zone/land", &z);
    let first = made(squash_with_hooks(&fx.land_git(), &refs(), ZONE, ACTOR, Some("Fix the guard")).unwrap());
    let log = fx.oplog();
    fx.install_hook("pre-commit", "#!/bin/sh\nprintf 'repeat output\n'\nprintf 'repeat error\n' >&2\nexit 7\n");

    let Err(HookedSquashError::Refused(refusal)) =
        squash_with_hooks(&fx.land_git(), &refs(), ZONE, ACTOR, Some("Fix the guard")) else {
            panic!("pre-commit must refuse the repeat");
        };
    assert_eq!(refusal, HookRefusal {
        hook: "pre-commit", code: Some(7), stdout: b"repeat output\n".to_vec(), stderr: b"repeat error\n".to_vec(),
    });
    assert_eq!(fx.ref_of(SQUASH_REF), Some(first.commit.to_string()));
    assert_eq!(fx.oplog(), log);
}

#[test]
fn a_commit_msg_hook_that_drops_the_trailers_is_unreadable_and_moves_nothing() {
    let fx = Fixture::new();
    let z = fx.commit_on(&fx.base, &[("z", "zone\n")], "add z", Some("r1"));
    fx.set_branch("zone/land", &z);
    fx.install_hook("commit-msg", "#!/bin/sh\nprintf 'subject\n' > \"$1\"\n");
    let log = fx.oplog();

    let Err(HookedSquashError::Unreadable { part, message }) =
        squash_with_hooks(&fx.land_git(), &refs(), ZONE, ACTOR, Some("Fix the guard")) else {
            panic!("missing trailers must be unreadable");
        };
    assert_eq!(part, "Mortar-Change-Id");
    assert_eq!(message, "subject\n");
    assert_eq!(fx.ref_of(SQUASH_REF), None);
    assert_eq!(fx.oplog(), log);
}

#[test]
fn a_commit_msg_hook_that_adds_a_malformed_change_id_is_unreadable_and_moves_nothing() {
    let fx = Fixture::new();
    let z = fx.commit_on(&fx.base, &[("z", "zone\n")], "add z", Some("r1"));
    fx.set_branch("zone/land", &z);
    fx.install_hook(
        "commit-msg",
        "#!/bin/sh\nawk '/^Mortar-Change-Id:/ { print \"Mortar-Change-Id: zzz\" } { print }' \"$1\" > \"$1.tmp\"\nmv \"$1.tmp\" \"$1\"\n",
    );
    let log = fx.oplog();

    let answer = squash_with_hooks(&fx.land_git(), &refs(), ZONE, ACTOR, Some("Fix the guard"));
    assert!(matches!(&answer, Err(HookedSquashError::Unreadable { .. })), "expected Unreadable, got {answer:?}");
    let Err(HookedSquashError::Unreadable { part, message }) = answer else { unreachable!() };
    assert_eq!(part, "Mortar-Change-Id");
    assert!(message.contains("Mortar-Change-Id: zzz\nMortar-Change-Id: "));
    assert!(message.ends_with(&format!("Mortar-Squash-Of: {z}\n")));
    assert_eq!(fx.ref_of(SQUASH_REF), None);
    assert_eq!(fx.oplog(), log);
}

#[test]
fn nothing_and_conflicted_run_no_hooks() {
    for conflict in [false, true] {
        let fx = Fixture::new();
        let expected = if conflict {
            let origin = fx.commit_on(&fx.base, &[("f", "origin\n")], "origin f", None);
            fx.set_branch("main", &origin);
            let zone = fx.commit_on(&fx.base, &[("f", "zone\n")], "zone f", Some("r1"));
            fx.set_branch("zone/land", &zone);
            SquashAnswer::Conflicted { files: vec!["f".into()] }
        } else {
            fx.set_branch("zone/land", &fx.base);
            SquashAnswer::Nothing { source: oid(&fx.base) }
        };
        fx.install_hook("pre-commit", "#!/bin/sh\nexit 7\n");
        fx.install_hook("commit-msg", "#!/bin/sh\nexit 9\n");

        let answer = squash_with_hooks(&fx.land_git(), &refs(), ZONE, ACTOR, None).unwrap();
        assert_eq!(answer, expected, "conflict={conflict}");
        assert_eq!(fx.ref_of(SQUASH_REF), None);
    }
}
