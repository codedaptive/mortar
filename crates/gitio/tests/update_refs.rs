//! Joins Git::update_refs -> stdin encoding -> Git::run -> real update-ref -> outcome classification.
//! Repository setup and observations use ordinary git commands, not update_refs itself.

#[path = "support/mod.rs"]
mod support;

use gitio::{Oid, RefUpdate, RunOpts, UpdateRefsOutcome};
use support::TestRepo;

#[test]
fn clean_multi_ref_transaction_moves_every_ref_to_its_requested_value() {
    let repo = TestRepo::new();
    let [first, second, third] = commits(&repo);
    seed_ref(&repo, "refs/heads/a", &first);
    seed_ref(&repo, "refs/heads/b", &second);
    assert_eq!(heads(&repo), format!("refs/heads/a {first}\nrefs/heads/b {second}\n"));

    let result = repo.git.update_refs(&[
        RefUpdate {
            refname: "refs/heads/a".into(),
            new: second.clone(),
            old: Some(first.clone()),
        },
        RefUpdate {
            refname: "refs/heads/b".into(),
            new: third.clone(),
            old: Some(second.clone()),
        },
        RefUpdate {
            refname: "refs/heads/c".into(),
            new: first.clone(),
            old: None,
        },
    ]).unwrap();

    assert_eq!(result, UpdateRefsOutcome::Updated);
    assert_eq!(heads(&repo), format!(
        "refs/heads/a {second}\nrefs/heads/b {third}\nrefs/heads/c {first}\n"
    ));
}

#[test]
fn stale_expected_value_names_the_ref_and_moves_nothing() {
    // Cover the three real git refusal wordings: wrong oid, already exists, and missing ref.
    for (case, actual, expected) in [
        ("wrong oid", Some(0), Some(1)),
        ("already exists", Some(0), None),
        ("missing ref", None, Some(0)),
    ] {
        let repo = TestRepo::new();
        let objects = commits(&repo);
        seed_ref(&repo, "refs/heads/a-good", &objects[0]);
        if let Some(index) = actual {
            seed_ref(&repo, "refs/heads/z-stale", &objects[index]);
        }
        let unchanged = match actual {
            Some(index) => format!(
                "refs/heads/a-good {}\nrefs/heads/z-stale {}\n", objects[0], objects[index]
            ),
            None => format!("refs/heads/a-good {}\n", objects[0]),
        };
        assert_eq!(heads(&repo), unchanged, "setup: {case}");

        // A valid update deliberately precedes the stale entry: an implementation that
        // writes refs one at a time would change a-good before discovering the refusal.
        let result = repo.git.update_refs(&[
            RefUpdate {
                refname: "refs/heads/a-good".into(),
                new: objects[2].clone(),
                old: Some(objects[0].clone()),
            },
            RefUpdate {
                refname: "refs/heads/z-stale".into(),
                new: objects[2].clone(),
                old: expected.map(|index| objects[index].clone()),
            },
            RefUpdate {
                refname: "refs/heads/zz-new".into(),
                new: objects[1].clone(),
                old: None,
            },
        ]).unwrap();

        assert_eq!(result, UpdateRefsOutcome::Stale {
            refname: "refs/heads/z-stale".into(),
        }, "{case}");
        assert_eq!(heads(&repo), unchanged, "partial transaction: {case}");
    }
}

fn commits(repo: &TestRepo) -> [Oid; 3] {
    let tree = repo.git.run(&["mktree"], &RunOpts::default()).unwrap();
    let tree = String::from_utf8(tree.stdout).unwrap();
    ["first\n", "second\n", "third\n"].map(|message| {
        let output = repo.git.run(&["commit-tree", tree.trim_end()], &RunOpts {
            stdin: Some(message.as_bytes().to_vec()),
            env: vec![
                ("GIT_AUTHOR_NAME".into(), "Mortar Integration".into()),
                ("GIT_AUTHOR_EMAIL".into(), "integration@example.invalid".into()),
                ("GIT_COMMITTER_NAME".into(), "Mortar Integration".into()),
                ("GIT_COMMITTER_EMAIL".into(), "integration@example.invalid".into()),
                ("GIT_AUTHOR_DATE".into(), "2000-01-01T00:00:00 +0000".into()),
                ("GIT_COMMITTER_DATE".into(), "2000-01-01T00:00:00 +0000".into()),
            ],
            ..RunOpts::default()
        }).unwrap();
        Oid::parse(String::from_utf8(output.stdout).unwrap().trim_end()).unwrap()
    })
}

fn seed_ref(repo: &TestRepo, name: &str, oid: &Oid) {
    repo.git.run(&["update-ref", name, oid.as_str()], &RunOpts::default()).unwrap();
}

fn heads(repo: &TestRepo) -> String {
    let output = repo.git.run(
        &["for-each-ref", "--sort=refname", "--format=%(refname) %(objectname)", "refs/heads/"],
        &RunOpts { read_only: true, ..RunOpts::default() },
    ).unwrap();
    String::from_utf8(output.stdout).unwrap()
}
