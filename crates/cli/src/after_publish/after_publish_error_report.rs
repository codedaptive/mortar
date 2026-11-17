//! land's after_publish refusals and failures in the cli's own words (SPEC-cli 06, S-128 (2)): land's, join's and
//! gitio's Display texts are never printed; only an error's refnames, zones and oids are. No report says that
//! nothing half-done is left: after_publish stops at a zone with the zones before it already moved.

use join::JoinError;
use land::AfterPublishError;

use super::after_publish_failure;
use crate::landing::{land_error_report, landing_refusal, squash_error_report};
use crate::report::Report;

const VERB: &str = "after-publish";
/// Every refusal's and failure's second line: after_publish stops at the zone it names, so the zones before it
/// moved, and the folder moves come only after land answered.
const SECOND: &str = "any zone named before it was already moved; no zone folder was brought to its tip";
/// The next of a zone commit replay cannot take.
const ONE_PARENT_NEXT: &str =
    "rewrite the zone's commits as one-parent commits with author dates, then repeat the same command";

/// Git and Join's Git and TipNotReplayed are failures (exit 5) naming what failed, then SECOND; Squash and Land are
/// worded as the landing verbs word them, Land with `integration_branch`, with SECOND in place of their "nothing
/// half-done is left"; every other arm is a refusal (exit 1) naming what stopped the zone, SECOND, its owner and a
/// next step.
pub fn after_publish_error_report(error: &AfterPublishError, integration_branch: &str) -> Report {
    match error {
        AfterPublishError::Git(_) => failed("git failed while reading or replaying a zone"),
        AfterPublishError::Squash(error) => with_second(squash_error_report(VERB, error)),
        AfterPublishError::Land(error) => with_second(land_error_report(VERB, error, integration_branch)),
        AfterPublishError::Join(error) => join_error_report(error),
        AfterPublishError::Unrelated { zone, origin, tip } => refused(
            format!("zone {zone} at {tip} shares no history with origin {origin}"),
            &format!("zone {zone}"),
            "rebuild the zone branch from origin, then repeat the same command".to_string(),
        ),
        AfterPublishError::NoSquashOf { zone } => refused(
            format!(
                "no squash of zone {zone} on origin's line carries a Mortar-Squash-Of trailer, so the squash \
                 published from it cannot be found"
            ),
            &format!("zone {zone}'s squashes"),
            "name the zone whose squash was published: mr after-publish <zone> [--move <zone>]... --actor <name>"
                .to_string(),
        ),
        AfterPublishError::SquashOfNotInZone { zone, squash_of } => refused(
            format!(
                "zone {zone}'s published squash was made from {squash_of}, which zone {zone} does not hold, itself \
                 or as a recorded replay"
            ),
            &format!("zone {zone}"),
            format!(
                "restore refs/heads/zone/{zone} to a commit that holds {squash_of} (find it with mr log --ref \
                 refs/heads/zone/{zone}, then mr restore <op id> refs/heads/zone/{zone} --actor <name>), then \
                 repeat the same command"
            ),
        ),
    }
}

/// A replay that stopped before it could answer clean or conflicted, arm by arm.
fn join_error_report(error: &JoinError) -> Report {
    let history = |line: String| refused(line, "the zone's history", ONE_PARENT_NEXT.to_string());
    match error {
        JoinError::Git(_) => failed("git failed while replaying a zone onto origin"),
        JoinError::OntoMissing(commit) => refused(
            format!("origin's commit {commit} does not exist here, so no zone can be replayed onto it"),
            "the repository's objects",
            format!("fetch or restore commit {commit}, then repeat the same command"),
        ),
        JoinError::TipMissing(refname) => refused(
            format!("{refname} does not exist, so its zone cannot be replayed"),
            "the repository's branches",
            format!("create {refname}, then repeat the same command"),
        ),
        JoinError::MergeCommit(commit) => history(format!("zone commit {commit} is a merge, so replay cannot take it")),
        JoinError::RootCommit(commit) => {
            history(format!("zone commit {commit} has no parent, so replay cannot take it"))
        }
        JoinError::EmptyAuthorDate(commit) => {
            history(format!("zone commit {commit} has an empty author date, so replay cannot take it"))
        }
        JoinError::TipNotReplayed(refname) => failed(&format!("git's replay gave no update for {refname}")),
    }
}

/// A refusal (exit 1): `line`, then SECOND, with its owner and next step.
fn refused(line: String, owner: &str, next: String) -> Report {
    landing_refusal(VERB, vec![line, SECOND.to_string()], owner, next)
}

/// A failure (exit 5): `what`, then SECOND.
fn failed(what: &str) -> Report {
    after_publish_failure(vec![what.to_string(), SECOND.to_string()])
}

/// A landing verb's report for after-publish: its "nothing half-done is left" line dropped, since zones before the
/// one that stopped may have moved, and SECOND added when it is not there.
fn with_second(mut report: Report) -> Report {
    report.lines.retain(|line| line != "nothing half-done is left");
    if !report.lines.iter().any(|line| line == SECOND) {
        report.lines.push(SECOND.to_string());
    }
    report
}
