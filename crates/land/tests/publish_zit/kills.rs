//! Case 3: a publish killed by SIGKILL between its two steps (the testkit crash shim, held by the kill gate rather
//! than timing), then oplog::recover, then publish again finishing once. The push runs no `git update-ref`, so the
//! shim's first hit is step 2's, before it moves the local origin: by then the push intent is closed done, both its
//! records marked with the remote `origin`. recover appends nothing for it, and the repeat writes no second
//! publish-push record; step 2 and the clean-up finish done (S-103).

use crate::fixture::{git_env, refs, state_text, Fixture, ACTOR, ID_A};
use gitio::Git;
use land::{publish, PublishAnswer};
use oplog::{read_op_log, recover, OpPhase, OpResult};
use std::os::unix::process::ExitStatusExt;
use std::path::PathBuf;
use testkit::{run_with_kill_gate, test_child, write_git_kill_shim};

const WORKER_DATE: &str = "2001-02-03T04:05:06+00:00";
const SHIM: &str = "git-crash-boundary.sh";

#[test]
fn killed_between_the_steps_recovers_and_the_repeat_publishes_once() {
    let (fx, squash) = Fixture::landed();
    write_git_kill_shim(&fx.repo.path.join(SHIM)).unwrap();
    let mut command = test_child("kills::publish_worker").unwrap();
    command.env("MORTAR_TEST_REPO", &fx.repo.path).env("MORTAR_TEST_CRASH", "before");
    let output = run_with_kill_gate(&mut command, &fx.repo.path).unwrap();
    assert_eq!(
        output.status.signal(),
        Some(9),
        "worker was not SIGKILLed before step 2: {:?}\nstdout: {}\nstderr: {}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(fx.remote_main(), Some(squash.commit.clone()));
    assert_eq!(fx.branch("main"), fx.base);
    assert_eq!(fx.landing_json(), Some(state_text(ID_A, &squash.commit, &fx.base, &squash.commit)));
    let push_done = vec![
        (OpPhase::Intent, None, Some("origin".to_string())),
        (OpPhase::Done, Some(OpResult::Moved), Some("origin".to_string())),
    ];
    assert_eq!(fx.push_log(), push_done);
    let recovered = recover(&fx.git, &fx.common).unwrap();
    assert_eq!(recovered.iter().filter(|record| record.kind == "publish-push").count(), 0, "{recovered:?}");
    assert_eq!(publish(&fx.git, &refs(), ACTOR).unwrap(), PublishAnswer::Published { origin: squash.commit.clone() });
    assert_eq!(fx.branch("main"), squash.commit);
    assert_eq!(fx.remote_main(), Some(squash.commit.clone()));
    assert_eq!(fx.landing_json(), None);
    assert_eq!(fx.push_log(), push_done);
    let records = read_op_log(&fx.common).unwrap().records;
    let tail: Vec<(&str, OpPhase)> =
        records.iter().rev().take(4).rev().map(|record| (record.kind.as_str(), record.phase)).collect();
    assert_eq!(
        tail,
        vec![
            ("publish-origin", OpPhase::Intent),
            ("publish-origin", OpPhase::Done),
            ("publish-squash", OpPhase::Intent),
            ("publish-squash", OpPhase::Done),
        ]
    );
}

#[test]
#[ignore = "SIGKILL subprocess entry point; run by the kill case through test_child"]
fn publish_worker() {
    let path = PathBuf::from(std::env::var("MORTAR_TEST_REPO").unwrap());
    let mut git = Git::new(&path);
    git.program = path.join(SHIM);
    git.env = git_env(WORKER_DATE);
    git.env.push(("MORTAR_TEST_WORKER_PID".into(), std::process::id().to_string().into()));
    let answer = publish(&git, &refs(), "worker@host");
    panic!("the crash shim returned without killing publish: {answer:?}");
}
