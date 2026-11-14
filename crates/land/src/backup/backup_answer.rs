//! backup_answer: what a backup run tells its caller, from its plan and the push's outcomes (SPEC-land 08).
//! backup_answer_tests cover it.

use crate::backup::{Backup, BackupError, PushPlan};
use gitio::{GitError, PushOutcome};

/// `outcomes` holds one outcome per `plan.push` entry, in that order. Pushed -> the local name is pushed;
/// UpToDate -> it is unchanged, after `plan.unchanged`; the first Refused -> Refused naming that backup ref.
/// A count that differs from the plan's is a refusal (Git) too: no outcome can be matched to its ref.
pub(crate) fn backup_answer(plan: PushPlan, outcomes: Vec<PushOutcome>) -> Result<Backup, BackupError> {
    if outcomes.len() != plan.push.len() {
        let message =
            format!("push answered {} outcomes for {} backup refs", outcomes.len(), plan.push.len());
        return Err(BackupError::Git(GitError::refusal(vec!["push".to_string()], message)));
    }
    let mut answer = Backup { pushed: Vec::new(), unchanged: plan.unchanged };
    for ((local, push_ref), outcome) in plan.push.into_iter().zip(outcomes) {
        match outcome {
            PushOutcome::Pushed => answer.pushed.push(local),
            PushOutcome::UpToDate => answer.unchanged.push(local),
            PushOutcome::Refused(refusal) => {
                return Err(BackupError::Refused { refname: push_ref.refname, refusal });
            }
        }
    }
    Ok(answer)
}
