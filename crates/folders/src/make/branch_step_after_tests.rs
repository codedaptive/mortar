//! branch_step_after: Done is Ready; a repeat is Ready at the tip, Remake when the branch is gone, Exists
//! elsewhere; Failed is Exists.

use gitio::Oid;
use oplog::{ApplyOutcome, OpPhase, OpRecord, OpResult};

use super::branch_step_after::branch_step_after;
use super::BranchStep;

const TIP: &str = "489d82745705a031306253da29bdad7aee19d2e5";
const OTHER: &str = "1111111111111111111111111111111111111111";

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

fn rec(phase: OpPhase, result: OpResult) -> OpRecord {
    OpRecord {
        op_id: "op-1".to_string(),
        key: format!("folders:branch:req/folders/WO-1@{TIP}"),
        kind: "folder-branch".to_string(),
        actor: "bilby".to_string(),
        reason: "make folder branch req/folders/WO-1".to_string(),
        at: 1,
        phase,
        moves: Vec::new(),
        result: Some(result),
    }
}

fn done() -> OpRecord {
    rec(OpPhase::Done, OpResult::Moved)
}

#[test]
fn done_is_ready() {
    let step = branch_step_after(&ApplyOutcome::Done(done()), None, &oid(TIP));
    assert_eq!(step, BranchStep::Ready);
}

#[test]
fn a_repeat_with_the_branch_at_the_tip_is_ready() {
    let step = branch_step_after(&ApplyOutcome::Repeated(done()), Some(&oid(TIP)), &oid(TIP));
    assert_eq!(step, BranchStep::Ready);
}

#[test]
fn a_repeat_with_the_branch_gone_is_remake() {
    let step = branch_step_after(&ApplyOutcome::Repeated(done()), None, &oid(TIP));
    assert_eq!(step, BranchStep::Remake);
}

#[test]
fn a_repeat_with_the_branch_elsewhere_exists() {
    let step = branch_step_after(&ApplyOutcome::Repeated(done()), Some(&oid(OTHER)), &oid(TIP));
    assert_eq!(step, BranchStep::Exists);
}

#[test]
fn failed_exists() {
    let failed = rec(
        OpPhase::Failed,
        OpResult::Stale {
            refname: "refs/heads/req/folders/WO-1".to_string(),
            actual: Some(oid(OTHER)),
        },
    );
    assert_eq!(branch_step_after(&ApplyOutcome::Failed(failed), None, &oid(TIP)), BranchStep::Exists);
}
