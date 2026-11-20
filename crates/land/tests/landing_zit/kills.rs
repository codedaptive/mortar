//! A merge landing killed by SIGKILL before or after its one update-ref (the testkit crash shim,
//! held by the kill gate rather than timing), then oplog::recover, then the same land again.
//! The worker commits with its own committer date, so a repeat that built the merge again would answer a different
//! commit from the one the landing state names.

use crate::fixture::{git_env, refs, Fixture, ACTOR, ID_A, ORIGIN_FILES, SQUASH_FILES};
use gitio::{Git, Oid};
use land::{land, LandAnswer, Squash};
use oplog::{oplog_path, read_op_log, recover, ChangeId, OpPhase};
use std::os::unix::process::ExitStatusExt;
use std::path::PathBuf;
use testkit::{run_with_kill_gate, test_child, write_git_kill_shim};

const WORKER_DATE: &str = "2001-02-03T04:05:06+00:00";
const SHIM: &str = "git-crash-boundary.sh";

/// A merge landing (origin moved past the squash's parent) run by the worker and killed at `stage`.
fn kill_merge_landing(stage: &str) -> (Fixture, Squash, Oid) {
    let fx = Fixture::new();
    let squash = fx.squash(&fx.base, &SQUASH_FILES, ID_A);
    let origin = fx.move_origin(&ORIGIN_FILES);
    write_git_kill_shim(&fx.repo.path.join(SHIM)).unwrap();
    let mut command = test_child("kills::land_worker").unwrap();
    command
        .env("MORTAR_TEST_REPO", &fx.repo.path)
        .env("MORTAR_TEST_SQUASH", squash.commit.as_str())
        .env("MORTAR_TEST_CHANGE_ID", squash.change_id.as_str())
        .env("MORTAR_TEST_SOURCE", squash.source.as_str())
        .env("MORTAR_TEST_CRASH", stage);
    let output = run_with_kill_gate(&mut command, &fx.repo.path).unwrap();
    assert_eq!(
        output.status.signal(),
        Some(9),
        "worker was not SIGKILLed at {stage}: {:?}\nstdout: {}\nstderr: {}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    (fx, squash, origin)
}

/// The integration commit landing.json names, after checking every other field against the fixture's values.
fn state_commit(fx: &Fixture, squash: &Squash, origin: &Oid) -> Oid {
    let text = fx.landing_json().expect("land writes landing.json before its op");
    let prefix = format!(
        "{{\"change_id\":\"{ID_A}\",\"zone\":\"land\",\"squash\":\"{}\",\"origin_at_start\":\"{origin}\",\
         \"integration\":\"",
        squash.commit
    );
    let hex = text.strip_prefix(&prefix).and_then(|rest| rest.strip_suffix("\"}\n"));
    Oid::parse(hex.unwrap_or_else(|| panic!("landing.json is not this landing's state: {text}"))).unwrap()
}

fn phases(fx: &Fixture) -> Vec<(String, OpPhase)> {
    read_op_log(&fx.common).unwrap().records.into_iter().map(|record| (record.kind, record.phase)).collect()
}

#[test]
fn killed_between_state_and_op_recovers_and_the_repeat_lands_the_states_commit() {
    let (fx, squash, origin) = kill_merge_landing("before");
    let merge = state_commit(&fx, &squash, &origin);
    assert_eq!(fx.git.commit_info(&merge).unwrap().parents, vec![origin.clone(), squash.commit.clone()]);
    assert_eq!(fx.branch("integration"), origin);
    assert_eq!(phases(&fx), vec![("land".to_string(), OpPhase::Intent)]);
    let closed = recover(&fx.git, &fx.common).unwrap();
    assert_eq!(closed.iter().map(|record| record.phase).collect::<Vec<_>>(), vec![OpPhase::Failed]);
    assert_eq!(land(&fx.git, &refs(), &squash, ACTOR).unwrap(), LandAnswer::Landed { integration: merge.clone() });
    assert_eq!(fx.branch("integration"), merge);
    assert_eq!(
        phases(&fx),
        [OpPhase::Intent, OpPhase::Failed, OpPhase::Intent, OpPhase::Done].map(|phase| ("land".to_string(), phase))
    );
}

#[test]
fn killed_after_the_op_recovers_done_and_the_repeat_is_landed_with_no_new_record() {
    let (fx, squash, origin) = kill_merge_landing("after");
    let merge = state_commit(&fx, &squash, &origin);
    assert_eq!(fx.git.commit_info(&merge).unwrap().parents, vec![origin.clone(), squash.commit.clone()]);
    assert_eq!(fx.branch("integration"), merge);
    assert_eq!(phases(&fx), vec![("land".to_string(), OpPhase::Intent)]);
    let closed = recover(&fx.git, &fx.common).unwrap();
    assert_eq!(closed.iter().map(|record| record.phase).collect::<Vec<_>>(), vec![OpPhase::Done]);
    let log = std::fs::read(oplog_path(&fx.common)).unwrap();
    assert_eq!(land(&fx.git, &refs(), &squash, ACTOR).unwrap(), LandAnswer::Landed { integration: merge.clone() });
    assert_eq!(fx.branch("integration"), merge);
    assert_eq!(std::fs::read(oplog_path(&fx.common)).unwrap(), log);
}

#[test]
#[ignore = "SIGKILL subprocess entry point; run by the kill cases through test_child"]
fn land_worker() {
    let var = |key: &str| std::env::var(key).unwrap();
    let path = PathBuf::from(var("MORTAR_TEST_REPO"));
    let squash = Squash {
        commit: Oid::parse(&var("MORTAR_TEST_SQUASH")).unwrap(),
        change_id: ChangeId::parse(&var("MORTAR_TEST_CHANGE_ID")).unwrap(),
        source: Oid::parse(&var("MORTAR_TEST_SOURCE")).unwrap(),
        requests: vec!["r1".to_string()],
    };
    let mut git = Git::new(&path);
    git.program = path.join(SHIM);
    git.env = git_env(WORKER_DATE);
    git.env.push(("MORTAR_TEST_WORKER_PID".into(), std::process::id().to_string().into()));
    let answer = land(&git, &refs(), &squash, "worker@host");
    panic!("the crash shim returned without killing land: {answer:?}");
}
