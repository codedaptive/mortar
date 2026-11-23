use crate::fixture::{git_at, git_env, refs, Fixture, ACTOR, RECEIPT};
use gitio::Oid;
use land::{commit_files, publish, CommitFilesAnswer, PublishAnswer};
use oplog::{recover, OpPhase};
use std::os::unix::process::ExitStatusExt;
use std::path::PathBuf;
use testkit::{run_with_kill_gate, test_child, write_git_kill_shim};

const SHIM: &str = "git-crash-boundary.sh";

fn kill(stage: &str) -> (Fixture, Oid) {
    let (fx, squash, _) = Fixture::landed();
    fx.receipt();
    write_git_kill_shim(&fx.repo.path.join(SHIM)).unwrap();
    let mut child = test_child("kills::commit_worker").unwrap();
    child.env("MORTAR_TEST_FOLDER", &fx.folder)
        .env("MORTAR_TEST_SHIM", fx.repo.path.join(SHIM))
        .env("MORTAR_TEST_CRASH", stage);
    let output = run_with_kill_gate(&mut child, &fx.repo.path).unwrap();
    assert_eq!(output.status.signal(), Some(9), "worker: {:?}\n{}\n{}", output.status,
        String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr));
    (fx, squash.commit)
}

fn pending(fx: &Fixture, landed: &Oid) -> Oid {
    let text = fx.state().unwrap();
    let prefix = fx.state_text(landed, landed, None).strip_suffix("}\n").unwrap().to_string();
    let hex = text.strip_prefix(&format!("{prefix},\"pending\":\"")).unwrap().strip_suffix("\"}\n").unwrap();
    let commit = Oid::parse(hex).unwrap();
    assert_eq!(text, fx.state_text(landed, landed, Some(&commit)));
    commit
}

#[test]
fn killed_before_update_ref_recovers_failed_then_repeat_moves_once() {
    let (fx, landed) = kill("before");
    let killed_commit = pending(&fx, &landed);
    fx.assert_commit(&killed_commit, &landed);
    assert_eq!(fx.branch("integration"), landed);
    assert_eq!(fx.phases(), vec![OpPhase::Intent]);
    assert_eq!(recover(&fx.git, &fx.common).unwrap().iter().map(|r| r.phase).collect::<Vec<_>>(), vec![OpPhase::Failed]);
    let git = git_at(fx.folder.clone());
    let CommitFilesAnswer::Committed { commit } = commit_files(&git, &refs(), &[PathBuf::from(RECEIPT)], "receipt\n", ACTOR).unwrap() else {
        panic!("repeat must move integration");
    };
    assert_eq!(fx.branch("integration"), commit);
    fx.assert_commit(&commit, &landed);
    assert_eq!(fx.phases(), vec![OpPhase::Intent, OpPhase::Failed, OpPhase::Intent, OpPhase::Done]);
    assert_eq!(fx.state(), Some(fx.state_text(&landed, &commit, None)));
}

#[test]
fn killed_after_update_ref_recovers_done_repeats_same_and_publishes() {
    let (fx, landed) = kill("after");
    let commit = pending(&fx, &landed);
    assert_eq!(fx.branch("integration"), commit);
    fx.assert_commit(&commit, &landed);
    assert_eq!(fx.phases(), vec![OpPhase::Intent]);
    assert_eq!(recover(&fx.git, &fx.common).unwrap().iter().map(|r| r.phase).collect::<Vec<_>>(), vec![OpPhase::Done]);
    let git = git_at(fx.folder.clone());
    assert_eq!(commit_files(&git, &refs(), &[PathBuf::from(RECEIPT)], "receipt\n", ACTOR).unwrap(), CommitFilesAnswer::Same);
    assert_eq!(fx.branch("integration"), commit);
    assert_eq!(fx.state(), Some(fx.state_text(&landed, &landed, Some(&commit))));
    assert_eq!(publish(&git, &refs(), ACTOR).unwrap(), PublishAnswer::Published { origin: commit.clone() });
    assert_eq!(fx.remote.read_ref("refs/heads/main").unwrap(), Some(commit));
}

#[test]
#[ignore = "SIGKILL subprocess entry point"]
fn commit_worker() {
    let mut git = git_at(PathBuf::from(std::env::var_os("MORTAR_TEST_FOLDER").unwrap()));
    git.program = PathBuf::from(std::env::var_os("MORTAR_TEST_SHIM").unwrap());
    git.env = git_env("2001-02-03T04:05:06+00:00");
    git.env.push(("MORTAR_TEST_WORKER_PID".into(), std::process::id().to_string().into()));
    let answer = commit_files(&git, &refs(), &[PathBuf::from(RECEIPT)], "receipt\n", ACTOR);
    panic!("crash shim returned without killing the worker: {answer:?}");
}
