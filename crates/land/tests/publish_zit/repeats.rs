//! Mixed-setting repeats: an open push intent refuses push-off; a push-off run killed after moving the local
//! origin finishes with push on without pushing. Both use throwaway local and remote repositories.

use crate::fixture::{git_env, refs, Fixture, ACTOR, OPEN_PUSH_OP_ID, SQUASH_REF};
use gitio::Git;
use land::{publish_with_push, PublishAnswer, PublishError, PublishOutcome};
use oplog::{read_op_log, recover, OpPhase};
use std::os::unix::process::ExitStatusExt;
use std::path::PathBuf;
use testkit::{run_with_kill_gate, test_child, write_git_kill_shim};

const SHIM: &str = "git-crash-boundary.sh";

#[test]
fn open_push_intent_refuses_push_off_then_push_on_finishes() {
    let (fx, squash) = Fixture::landed();
    fx.open_push_intent(&squash.commit);
    let landing = fx.landing_json();
    assert!(landing.is_some());
    let records = read_op_log(&fx.common).unwrap().records.len();

    let result = publish_with_push(&fx.git, &refs(), ACTOR, false);
    assert!(matches!(&result, Err(PublishError::PushIntent { op_id, phase: OpPhase::Intent })
        if op_id == OPEN_PUSH_OP_ID), "{result:?}");
    assert_eq!(fx.branch("main"), fx.base);
    assert_eq!(fx.branch("integration"), squash.commit);
    assert_eq!(fx.landing_json(), landing);
    assert_eq!(fx.git.read_ref(SQUASH_REF).unwrap(), Some(squash.commit.clone()));
    assert_eq!(read_op_log(&fx.common).unwrap().records.len(), records);
    assert_eq!(fx.remote_main(), Some(fx.base.clone()));

    assert_eq!(
        publish_with_push(&fx.git, &refs(), ACTOR, true).unwrap(),
        PublishOutcome { answer: PublishAnswer::Published { origin: squash.commit.clone() }, pushed: true }
    );
    assert_eq!(fx.branch("main"), squash.commit);
    assert_eq!(fx.remote_main(), Some(squash.commit));
}

#[test]
fn push_off_killed_after_origin_move_finishes_push_on_without_push() {
    let (fx, squash) = Fixture::landed();
    write_git_kill_shim(&fx.repo.path.join(SHIM)).unwrap();
    let mut command = test_child("repeats::push_off_worker").unwrap();
    command.env("MORTAR_TEST_REPO", &fx.repo.path).env("MORTAR_TEST_CRASH", "after");
    let output = run_with_kill_gate(&mut command, &fx.repo.path).unwrap();
    assert_eq!(output.status.signal(), Some(9), "worker was not killed after update-ref: {output:?}");
    assert_eq!(fx.branch("main"), squash.commit);
    assert!(fx.landing_json().is_some());
    assert_eq!(fx.git.read_ref(SQUASH_REF).unwrap(), Some(squash.commit.clone()));
    assert_eq!(fx.remote_main(), Some(fx.base.clone()));
    assert!(fx.push_log().is_empty());

    recover(&fx.git, &fx.common).unwrap();
    assert_eq!(
        publish_with_push(&fx.git, &refs(), ACTOR, true).unwrap(),
        PublishOutcome { answer: PublishAnswer::Published { origin: squash.commit.clone() }, pushed: false }
    );
    assert_eq!(fx.branch("main"), squash.commit);
    assert_eq!(fx.remote_main(), Some(fx.base.clone()));
    assert!(fx.push_log().is_empty());
    assert_eq!(fx.landing_json(), None);
    assert_eq!(fx.git.read_ref(SQUASH_REF).unwrap(), None);
}

#[test]
#[ignore = "SIGKILL subprocess entry point; run by the push-off kill case"]
fn push_off_worker() {
    let path = PathBuf::from(std::env::var("MORTAR_TEST_REPO").unwrap());
    let mut git = Git::new(&path);
    git.program = path.join(SHIM);
    git.env = git_env("2001-02-03T04:05:06+00:00");
    git.env.push(("MORTAR_TEST_WORKER_PID".into(), std::process::id().to_string().into()));
    let answer = publish_with_push(&git, &refs(), "worker@host", false);
    panic!("the crash shim returned without killing publish: {answer:?}");
}
