//! backup_answer: each push outcome's place in the answer, the first refusal, and a count that does not match.

use super::{backup_answer, Backup, BackupError, PushPlan};
use gitio::{Oid, PushLease, PushOutcome, PushRef, PushRefusal};

const A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

fn set(local: &str, backup: &str) -> (String, PushRef) {
    let new = Some(Oid::parse(A).unwrap());
    (local.to_string(), PushRef { refname: backup.to_string(), new, lease: PushLease::Absent })
}

/// Three refs to push and one the plan already found unchanged.
fn plan() -> PushPlan {
    PushPlan {
        push: vec![
            set("refs/heads/zone/a", "refs/mortar-backup/h1/heads/zone/a"),
            set("refs/heads/req/r", "refs/mortar-backup/h1/heads/req/r"),
            set("refs/mortar/snap/s", "refs/mortar-backup/h1/mortar/snap/s"),
        ],
        unchanged: vec!["refs/mortar/squash/land".to_string()],
    }
}

#[test]
fn pushed_refs_are_pushed_and_up_to_date_ones_follow_the_plans_unchanged() {
    let outcomes = vec![PushOutcome::Pushed, PushOutcome::UpToDate, PushOutcome::Pushed];
    let expected = Backup {
        pushed: vec!["refs/heads/zone/a".to_string(), "refs/mortar/snap/s".to_string()],
        unchanged: vec!["refs/mortar/squash/land".to_string(), "refs/heads/req/r".to_string()],
    };
    assert_eq!(backup_answer(plan(), outcomes).ok(), Some(expected));
}

#[test]
fn an_empty_plan_answers_the_plans_unchanged() {
    let plan = PushPlan { push: vec![], unchanged: vec!["refs/heads/zone/a".to_string()] };
    let expected = Backup { pushed: vec![], unchanged: vec!["refs/heads/zone/a".to_string()] };
    assert_eq!(backup_answer(plan, vec![]).ok(), Some(expected));
}

#[test]
fn the_first_refusal_names_its_backup_ref_and_why() {
    let outcomes = vec![
        PushOutcome::Pushed,
        PushOutcome::Refused(PushRefusal::StaleLease),
        PushOutcome::Refused(PushRefusal::Hook),
    ];
    match backup_answer(plan(), outcomes) {
        Err(BackupError::Refused { refname, refusal }) => {
            assert_eq!(refname, "refs/mortar-backup/h1/heads/req/r");
            assert_eq!(refusal, PushRefusal::StaleLease);
        }
        other => panic!("expected Refused, got {other:?}"),
    }
}

#[test]
fn fewer_outcomes_than_refs_is_a_git_refusal() {
    match backup_answer(plan(), vec![PushOutcome::Pushed, PushOutcome::Pushed]) {
        Err(BackupError::Git(error)) => {
            assert_eq!(error.code, None);
            assert_eq!(error.stderr, "push answered 2 outcomes for 3 backup refs");
        }
        other => panic!("expected Git, got {other:?}"),
    }
}

#[test]
fn more_outcomes_than_refs_is_a_git_refusal() {
    let outcomes = vec![PushOutcome::Pushed; 4];
    match backup_answer(plan(), outcomes) {
        Err(BackupError::Git(error)) => assert_eq!(error.stderr, "push answered 4 outcomes for 3 backup refs"),
        other => panic!("expected Git, got {other:?}"),
    }
}
