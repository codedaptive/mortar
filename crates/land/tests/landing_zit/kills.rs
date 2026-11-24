//! A merge landing killed by SIGKILL before or after its update-ref, or a batch's second member killed before its
//! update-ref (the testkit crash shim, held by the kill gate rather than timing), then oplog::recover, then the same
//! land again, or back_out of the batch.
//! The worker commits with its own committer date, so a repeat that built the merge again would answer a different
//! commit from the one the landing state names.

use crate::fixture::{git_env, refs, state_text, tree, Fixture, ACTOR, ID_A, ID_B, MERGED_FILES, ORIGIN_FILES, SQUASH_FILES};
use gitio::{Git, Oid};
use land::{back_out_batch, land, land_batch, BatchBackOutAnswer, BatchLandAnswer, LandAnswer, Squash};
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

/// The merge commit the landing built, rebuilt here with the worker's committer date, after checking landing.json is
/// exactly the one-landing state text naming it and the fixture's other values.
fn state_commit(fx: &Fixture, squash: &Squash, origin: &Oid) -> Oid {
    let mut git = Git::new(&fx.repo.path);
    git.env = git_env(WORKER_DATE);
    let merge = git.commit_tree(&tree(&git, &MERGED_FILES), &[origin.clone(), squash.commit.clone()],
        &format!("land {ID_A} (land)\n"), None, None).unwrap();
    assert_eq!(fx.landing_json(), Some(state_text(ID_A, &squash.commit, origin, &merge)));
    merge
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

fn kill_second_member() -> (Fixture, Squash, Oid) {
    let fx = Fixture::new();
    let first = fx.squash(&fx.base, &SQUASH_FILES, ID_A);
    assert_eq!(land_batch(&fx.git, &refs(), &first, ACTOR).unwrap(), BatchLandAnswer::Landed { integration: first.commit.clone() });
    let message = format!("other squash\n\nMortar-Change-Id: {ID_B}\nMortar-Zone: other\nMortar-Request: r1\n");
    let second = Squash {
        commit: fx.git.commit_tree(&tree(&fx.git, &[("a.txt", "base\n"), ("other.txt", "other\n")]),
            std::slice::from_ref(&fx.base), &message, None, None).unwrap(),
        change_id: ChangeId::parse(ID_B).unwrap(), source: fx.base.clone(), requests: vec!["r1".into()],
    };
    write_git_kill_shim(&fx.repo.path.join(SHIM)).unwrap();
    let mut command = test_child("kills::land_worker").unwrap();
    command.env("MORTAR_TEST_REPO", &fx.repo.path)
        .env("MORTAR_TEST_SQUASH", second.commit.as_str())
        .env("MORTAR_TEST_CHANGE_ID", second.change_id.as_str())
        .env("MORTAR_TEST_SOURCE", second.source.as_str())
        .env("MORTAR_TEST_CRASH", "before");
    let output = run_with_kill_gate(&mut command, &fx.repo.path).unwrap();
    assert_eq!(output.status.signal(), Some(9), "worker stderr: {}", String::from_utf8_lossy(&output.stderr));
    let text = fx.landing_json().unwrap();
    let prefix = format!("{{\"origin_at_start\":\"{}\",\"landings\":[{{\"change_id\":\"{ID_A}\",\"zone\":\"land\",\"squash\":\"{}\",\"integration\":\"{}\"}},{{\"change_id\":\"{ID_B}\",\"zone\":\"other\",\"squash\":\"{}\",\"integration\":\"", fx.base, first.commit, first.commit, second.commit);
    let merge = Oid::parse(text.strip_prefix(&prefix).and_then(|s| s.strip_suffix("\"}]}\n")).unwrap()).unwrap();
    assert_eq!(fx.git.commit_info(&merge).unwrap().parents, vec![first.commit.clone(), second.commit.clone()]);
    assert_eq!(fx.branch("integration"), first.commit);
    (fx, second, merge)
}

#[test]
fn killed_second_member_recovers_and_repeat_lands_once() {
    let (fx, second, merge) = kill_second_member();
    recover(&fx.git, &fx.common).unwrap();
    assert_eq!(land_batch(&fx.git, &refs(), &second, ACTOR).unwrap(), BatchLandAnswer::Landed { integration: merge.clone() });
    assert_eq!(fx.branch("integration"), merge);
    let done = read_op_log(&fx.common).unwrap().records.into_iter().filter(|r|
        r.kind == "land" && r.phase == OpPhase::Done && r.reason.starts_with(&format!("land {ID_B} "))).count();
    assert_eq!(done, 1);
}

#[test]
fn killed_second_member_recovers_then_back_out_discards_both() {
    let (fx, _, _) = kill_second_member();
    recover(&fx.git, &fx.common).unwrap();
    assert_eq!(back_out_batch(&fx.git, &refs(), ACTOR).unwrap(), BatchBackOutAnswer::BackedOut {
        change_id: ChangeId::parse(ID_B).unwrap(),
        change_ids: vec![ChangeId::parse(ID_A).unwrap(), ChangeId::parse(ID_B).unwrap()],
    });
    assert_eq!(fx.branch("integration"), fx.base);
    assert_eq!(fx.landing_json(), None);
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
    let answer = land_batch(&git, &refs(), &squash, "worker@host");
    panic!("the crash shim returned without killing land: {answer:?}");
}
