use crate::fixture::{refs, run, Fixture, ACTOR, RECEIPT};
use gitio::HookRefusal;
use land::{commit_files_with_hooks, CommitFilesAnswer, HookedCommitFilesError};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

fn install_hook(fx: &Fixture, name: &str, script: &str) {
    let hooks = fx.common.join("hooks");
    fs::create_dir_all(&hooks).unwrap();
    let path = hooks.join(name);
    fs::write(&path, script).unwrap();
    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).unwrap();
}

fn refusing_hook(name: &'static str) {
    let (fx, squash, git) = Fixture::landed();
    fx.receipt();
    install_hook(&fx, name, "#!/bin/sh\nprintf 'hook output\n'\nprintf 'hook error\n' >&2\nexit 3\n");
    let state = fx.state();
    let log = fx.log();
    let index = run(&git, &["ls-files", "--stage"]);
    let staged = run(&git, &["diff", "--cached", "--name-only"]);

    let answer = commit_files_with_hooks(&git, &refs(), &[PathBuf::from(RECEIPT)], "receipt\n", ACTOR);
    assert!(matches!(&answer, Err(HookedCommitFilesError::Refused(_))), "{name} must refuse, got {answer:?}");
    let Err(HookedCommitFilesError::Refused(refusal)) = answer else { unreachable!() };
    assert_eq!(refusal, HookRefusal {
        hook: name, code: Some(3), stdout: b"hook output\n".to_vec(), stderr: b"hook error\n".to_vec(),
    });
    assert_eq!(fx.state(), state);
    assert_eq!(fx.log(), log);
    assert_eq!(fx.branch("integration"), squash.commit);
    assert_eq!(run(&git, &["ls-files", "--stage"]), index);
    assert_eq!(run(&git, &["diff", "--cached", "--name-only"]), staged);
}

#[test]
fn pre_commit_refusal_leaves_landing_and_index_unchanged() {
    refusing_hook("pre-commit");
}

#[test]
fn commit_msg_refusal_leaves_landing_and_index_unchanged() {
    refusing_hook("commit-msg");
}

#[test]
fn commit_msg_rewrite_becomes_the_committed_message() {
    let (fx, _squash, git) = Fixture::landed();
    fx.receipt();
    install_hook(&fx, "commit-msg", "#!/bin/sh\nprintf 'rewritten by hook\n' > \"$1\"\n");

    let answer = commit_files_with_hooks(&git, &refs(), &[PathBuf::from(RECEIPT)], "receipt\n", ACTOR).unwrap();
    let CommitFilesAnswer::Committed { commit } = answer else { panic!("changed path must be committed") };
    assert_eq!(fx.branch("integration"), commit);
    assert_eq!(git.commit_info(&commit).unwrap().message, "rewritten by hook\n");
}

#[test]
fn same_content_skips_refusing_pre_commit() {
    let (fx, squash, git) = Fixture::landed();
    install_hook(&fx, "pre-commit", "#!/bin/sh\nexit 3\n");
    let state = fx.state();
    let log = fx.log();

    assert_eq!(commit_files_with_hooks(&git, &refs(), &[PathBuf::from("a.txt")], "same\n", ACTOR).unwrap(), CommitFilesAnswer::Same);
    assert_eq!(fx.state(), state);
    assert_eq!(fx.log(), log);
    assert_eq!(fx.branch("integration"), squash.commit);
}
