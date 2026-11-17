//! land's bisect refusals and failures in the cli's own words (SPEC-cli 06).

use land::{BisectError, CulpritError};

use crate::landing::landing_refusal;
use crate::report::{failure_report, Report};

const VERB: &str = "bisect";

/// Each BisectError arm as a refusal (exit 1) naming the commits or ref involved and a next step, or, for git and
/// unreadable squash trailers, a failure (exit 5). No error text of gitio, oplog or the probe's spawn is printed:
/// ProbeCouldNotRun's `error` is an OS message, so only its commit is named.
pub fn bisect_error_report(error: &BisectError) -> Report {
    match error {
        BisectError::OriginMissing { refname } => landing_refusal(
            VERB,
            vec![format!("{refname} does not exist, so bisect has no origin line to search")],
            "the repository's branches",
            format!("create {refname}, then repeat the same command"),
        ),
        BisectError::NotOnOrigin { failed, origin_branch } => landing_refusal(
            VERB,
            vec![format!(
                "the failed commit {failed} is not on {origin_branch}'s first-parent line, so it names no landing"
            )],
            "the commits named",
            format!(
                "name a failed commit on {origin_branch}'s first-parent line: mr bisect <last pass> <failed> \
                 --timeout <seconds> --actor <name> -- <command>..."
            ),
        ),
        BisectError::OffTheLine { last_pass, failed } => landing_refusal(
            VERB,
            vec![format!(
                "the last passing commit {last_pass} is not on {failed}'s first-parent line, so the range names no \
                 landings"
            )],
            "the commits named",
            format!(
                "name a last passing commit on {failed}'s first-parent line: mr bisect <last pass> {failed} \
                 --timeout <seconds> --actor <name> -- <command>..."
            ),
        ),
        BisectError::ProbeCouldNotRun { commit, error: _ } => landing_refusal(
            VERB,
            vec![format!(
                "the test command could not be run at {commit}, so its verdict there is unknown and the search stopped"
            )],
            "the test command",
            "check that the test command's program exists and can be run, then repeat the same command".to_string(),
        ),
        BisectError::Git(_) => failure_report(VERB, "git failed while reading origin's line or probing a commit"),
        BisectError::Culprit(CulpritError::NotALanding { commit, parents }) => landing_refusal(
            VERB,
            vec![format!(
                "the first failing commit {commit} has {parents} parents, so it is not a landing (a landing has one \
                 or two)"
            )],
            &format!("the commit {commit}"),
            format!("inspect commit {commit} by hand: Mortar did not land it, so it names no squash"),
        ),
        BisectError::Culprit(CulpritError::Trailers(_)) => {
            failure_report(VERB, "the first failing landing's squash trailers could not be read")
        }
    }
}
