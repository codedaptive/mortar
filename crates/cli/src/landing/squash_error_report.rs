//! The report of a landing verb that land's squash (or a squash step inside land or publish) refused, in the cli's
//! own words: land's, gitio's and oplog's Display texts are never printed; only an error's refnames, oids, op ids,
//! keys and trailer names and values are.

use land::SquashError;

use super::landing_refusal;
use crate::report::{failure_report, Report};

/// - Git, Oplog, Trailers: failure_report (exit 5) naming what failed.
/// - ChangeId: refused, a malformed Mortar-Change-Id or no new change ID; the next step in words.
/// - RefMissing, AttrSourceMissing, Unrelated: refused, naming the ref or commits.
/// - OpFailed, OpRepeated: refused, naming the record's op id or key; the next step is a repeat.
/// - TrailerMissing: refused, naming the commit and the trailer; the next step is to name the commit at the zone's
///   squash ref, or to make the squash again.
/// - TrailerMalformed: refused, naming the commit, the trailer and its value; the next step is to make the squash
///   again.
/// - RequestsMalformed: refused, naming the commit and how squash reads a message's request lines (a squash with
///   or without a caller's message); the next step is to make the squash again.
///
/// Every refusal is exit 3, outcome "refused", with no fields.
pub fn squash_error_report(verb: &str, error: &SquashError) -> Report {
    let refused = |line: String, owner: &str, next: String| landing_refusal(verb, vec![line], owner, next);
    match error {
        SquashError::Git(_) => {
            failure_report(verb, "git failed while reading the branches or writing the squash commit")
        }
        SquashError::Oplog(_) => failure_report(verb, "the op log could not be read or written"),
        SquashError::Trailers(_) => {
            failure_report(verb, "the Mortar trailers of a squash or zone commit could not be read")
        }
        SquashError::ChangeId(_) => refused(
            "a zone commit carries a malformed Mortar-Change-Id, or no new change ID could be drawn".to_string(),
            "the zone branch's commits",
            "read the zone's Mortar-Change-Id trailers with git log, mend the malformed one through a request, then \
             repeat the same command"
                .to_string(),
        ),
        SquashError::RefMissing { refname } => refused(
            format!("{refname} does not exist; nothing was written and no ref moved"),
            "the repository's branches",
            format!("create {refname}, then repeat the same command"),
        ),
        SquashError::AttrSourceMissing { commit } => refused(
            format!("commit {commit}, the merge's attribute source, does not exist; nothing was written"),
            "the repository's objects",
            format!("fetch or restore commit {commit}, then repeat the same command"),
        ),
        SquashError::Unrelated { origin, source } => refused(
            format!("the zone tip {source} shares no history with origin {origin}; nothing was squashed"),
            "the zone branch",
            "rebuild the zone branch from origin, then repeat the same command".to_string(),
        ),
        SquashError::OpFailed(record) => refused(
            format!("op {} moved nothing: the squash ref was not at its expected value", record.op_id),
            "the squash ref",
            "repeat the same command; it reads the refs afresh".to_string(),
        ),
        SquashError::OpRepeated(record) => refused(
            format!("op key {} is already done in the op log, so nothing moved", record.key),
            "the op log",
            "repeat the same command".to_string(),
        ),
        SquashError::TrailerMissing { commit, trailer } => refused(
            format!("commit {commit} carries no {trailer} trailer, so it is not a squash Mortar wrote"),
            &format!("the commit {commit}"),
            "name the commit at refs/mortar/squash/<zone>, or make the squash again: mr squash <zone> --actor \
             <name>"
                .to_string(),
        ),
        SquashError::TrailerMalformed { commit, trailer, value } => refused(
            format!("commit {commit}'s {trailer} trailer holds {value:?}, which is not a change ID or a commit id"),
            &format!("the commit {commit}"),
            "make the squash again: mr squash <zone> --actor <name>".to_string(),
        ),
        SquashError::RequestsMalformed { commit } => refused(
            format!(
                "commit {commit}'s message does not list its requests as squash reads them: one \"- <request>: ...\" \
                 line each in the paragraph before the trailers, as many as a \"<zone>: <n> requests\" subject counts"
            ),
            &format!("the commit {commit}"),
            "make the squash again: mr squash <zone> --actor <name>".to_string(),
        ),
    }
}
