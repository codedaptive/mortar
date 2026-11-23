//! Landing status after a publish is killed inside its push, and with no publish in progress, using throwaway
//! local and remote repositories.

use crate::fixture::{git_env, refs, Fixture, ACTOR, ID_A};
use gitio::Git;
use land::{publish, read_landing_status, LandingStatus, OpenPushIntent, PublishAnswer};
use oplog::{recover, ChangeId, OpPhase};
use std::os::unix::process::ExitStatusExt;
use std::path::PathBuf;
use testkit::{run_with_kill_gate, test_child, write_git_shim};

const SHIM: &str = "git-push-crash.sh";
// Testkit's shim fires only on update-ref, after the push intent is closed.
const PUSH_KILL_SHIM: &[u8] = br#"#!/bin/sh
set -eu
if [ "${3-}" != push ]; then exec git "$@"; fi
[ "$PPID" = "$MORTAR_TEST_WORKER_PID" ] || exit 98
git "$@"
printf 'armed\n' > "$TESTKIT_GATE_ARMED"
kill -KILL "$MORTAR_TEST_WORKER_PID"
# Keep our output pipes open until the outer test has observed and reaped worker death.
IFS= read -r release < "$TESTKIT_GATE_RELEASE"
[ "$release" = release ] || exit 97
# A live worker or zombie still has a pid; only parent reaping removes it.
if kill -0 "$MORTAR_TEST_WORKER_PID" 2>/dev/null; then
  printf 'not-reaped\n' > "$TESTKIT_GATE_ACK"
else
  printf 'reaped\n' > "$TESTKIT_GATE_ACK"
fi
"#;

#[test]
fn killed_inside_the_push_the_landing_status_names_the_open_intent() {
    let (fx, squash) = Fixture::landed();
    write_git_shim(&fx.repo.path.join(SHIM), PUSH_KILL_SHIM).unwrap();
    let mut command = test_child("status::push_worker").unwrap();
    command.env("MORTAR_TEST_REPO", &fx.repo.path);
    let output = run_with_kill_gate(&mut command, &fx.repo.path).unwrap();
    assert_eq!(
        output.status.signal(),
        Some(9),
        "worker was not SIGKILLed inside push: {:?}\nstdout: {}\nstderr: {}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(fx.remote_main(), Some(squash.commit.clone()));
    assert_eq!(fx.branch("main"), fx.base);
    assert_eq!(fx.push_log(), vec![(OpPhase::Intent, None, Some("origin".to_string()))]);

    recover(&fx.git, &fx.common).unwrap();
    let op_id = fx.push_records()[0].record.op_id.clone();
    let status = read_landing_status(&fx.git, &refs()).unwrap();
    assert_eq!(
        status,
        Some(LandingStatus {
            change_id: ChangeId::parse(ID_A).unwrap(),
            zone: "land".into(),
            squash: squash.commit.clone(),
            origin_at_start: fx.base.clone(),
            integration: squash.commit.clone(),
            open_push: Some(OpenPushIntent { op_id: op_id.clone() }),
        })
    );
    assert_eq!(
        status.unwrap().open_push.unwrap().to_string(),
        format!("push intent {op_id} is still open: a publish was interrupted; next: mr publish")
    );
    assert_eq!(publish(&fx.git, &refs(), ACTOR).unwrap(), PublishAnswer::Published { origin: squash.commit.clone() });
    assert_eq!(read_landing_status(&fx.git, &refs()).unwrap(), None);
}

#[test]
fn with_no_publish_in_progress_the_landing_status_carries_no_intent() {
    let fx = Fixture::new();
    assert_eq!(read_landing_status(&fx.git, &refs()).unwrap(), None);

    let (fx, squash) = Fixture::landed();
    assert_eq!(
        read_landing_status(&fx.git, &refs()).unwrap(),
        Some(LandingStatus {
            change_id: ChangeId::parse(ID_A).unwrap(),
            zone: "land".into(),
            squash: squash.commit.clone(),
            origin_at_start: fx.base.clone(),
            integration: squash.commit.clone(),
            open_push: None,
        })
    );
}

#[test]
#[ignore = "SIGKILL subprocess entry point; run by the push kill case through test_child"]
fn push_worker() {
    let path = PathBuf::from(std::env::var("MORTAR_TEST_REPO").unwrap());
    let mut git = Git::new(&path);
    git.program = path.join(SHIM);
    git.env = git_env("2001-02-03T04:05:06+00:00");
    git.env.push(("MORTAR_TEST_WORKER_PID".into(), std::process::id().to_string().into()));
    let answer = publish(&git, &refs(), "worker@host");
    panic!("the crash shim returned without killing publish: {answer:?}");
}
