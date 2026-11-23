//! The report of an `mr publish` that land's publish refused, in the cli's own words: land's, gitio's and
//! oplog's Display texts are never printed; only an error's refnames, remotes and op ids, and git's summary of a
//! push refusal it did not classify, are.

use gitio::PushRefusal;
use land::{LandRefs, PublishError};
use oplog::OpPhase;

use super::{land_error_report, landing_refusal, landing_state_error_report, squash_error_report};
use crate::report::{failure_report, Report};

const VERB: &str = "publish";
const PUBLISH: &str = "mr publish --actor <name>";

/// - Squash, Land, State: squash_error_report, land_error_report (with `refs`' integration branch) and
///   landing_state_error_report, as "publish".
/// - Git, Oplog: failure_report (exit 5) naming what failed.
/// - PushRefused: refused, why the remote <refs.remote> refused the push and that its branch is still at the
///   landing's starting origin; the next step clears the refusal, then publishes again.
/// - RemoteMissing: refused, nothing pushed; the next step creates the branch on the remote.
/// - PushIntent, phase Intent (a publish stopped between its steps; push off is refused while it is open): refused,
///   naming the op id, nothing pushed or changed; the next step is a plain publish, which reads the remote and
///   closes the intent. Any other phase, as land reads it (already done when publish opened it): refused, naming
///   the op id, nothing pushed; the next step checks the remote's origin branch, then publishes.
///
/// Every refusal is exit 1, outcome "refused", with no fields.
pub fn publish_error_report(error: &PublishError, refs: &LandRefs) -> Report {
    match error {
        PublishError::Squash(error) => squash_error_report(VERB, error),
        PublishError::Land(error) => land_error_report(VERB, error, &refs.integration_branch),
        PublishError::State(error) => landing_state_error_report(VERB, error),
        PublishError::Git(_) => {
            failure_report(VERB, "git failed while reading or pushing to the remote, or reading a ref")
        }
        PublishError::Oplog(_) => failure_report(VERB, "the op log could not be written"),
        PublishError::PushRefused { refname, refusal } => landing_refusal(
            VERB,
            vec![
                format!("the remote {} refused the push of {refname}: {}", refs.remote, refusal_words(refusal)),
                format!("the remote's {refname} is still at the landing's starting origin"),
            ],
            &format!("the remote {}", refs.remote),
            format!("clear what the remote refused, then {PUBLISH}"),
        ),
        PublishError::RemoteMissing { remote, refname } => landing_refusal(
            VERB,
            vec![format!("{remote} has no {refname} to publish onto; nothing was pushed")],
            &format!("the remote {remote}"),
            format!("create {refname} on {remote}, then {PUBLISH}"),
        ),
        PublishError::PushIntent { op_id, phase: OpPhase::Intent } => landing_refusal(
            VERB,
            vec![format!(
                "the push intent {op_id} for this move is still open: a publish stopped between its steps, so nothing \
                 was pushed or changed"
            )],
            "the op log",
            format!("{PUBLISH}: a plain publish reads the remote and closes the intent"),
        ),
        PublishError::PushIntent { op_id, .. } => landing_refusal(
            VERB,
            vec![format!(
                "the push intent {op_id} for this move was already done when publish opened it, so nothing was pushed"
            )],
            "the op log",
            format!("check {}'s {}, then {PUBLISH}", refs.remote, refs.origin_branch),
        ),
    }
}

/// Why the remote refused, in the cli's words; an unclassified refusal quotes git's summary.
fn refusal_words(refusal: &PushRefusal) -> String {
    match refusal {
        PushRefusal::StaleLease => "its value was not the one leased".to_string(),
        PushRefusal::NonFastForward => "it is not a fast-forward".to_string(),
        PushRefusal::Hook => "a hook on the remote declined it".to_string(),
        PushRefusal::Other(summary) => format!("git's summary was {summary}"),
    }
}
