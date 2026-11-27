//! resolve_squash: the squash a landing verb was told to read, by its commit or by the change ID its squash ref
//! carries. The one file in the cli that calls land::read_squash.
//! No sibling test: it runs git and land's read_squash for real; pick_squash_ref_tests,
//! squash_pick_report_tests and squash_error_report_tests cover its decisions.
//! The integration tests in crates/cli/tests/land_runs/land_flow/would_merge.rs cover it through the binary:
//! would-merge yes and no, by change ID and by commit; a change ID no squash ref carries, exit 3; two squash refs
//! carrying one change ID, exit 3 naming both; a commit read_squash refuses (no Mortar trailers), exit 3; a squash
//! ref holding such a commit beside the one carrying the change ID is passed over.

use gitio::{Git, Oid};
use land::{read_squash, Squash};
use oplog::ChangeId;

use super::{pick_squash_ref, squash_error_report, squash_pick_report, SquashName};
use crate::report::{failure_report, Report};

const SQUASH_REFS: &str = "refs/mortar/squash/";

/// - Commit: read_squash of that commit; a refusal is squash_error_report's.
/// - Change: every ref under refs/mortar/squash/ read with read_squash (a ref that does not read as a squash carries
///   no change ID), then pick_squash_ref; Missing and Several are squash_pick_report's refusals, One is the squash
///   read from that ref. A failed listing is failure_report (exit 5).
pub fn resolve_squash(git: &Git, verb: &str, name: &SquashName) -> Result<Squash, Report> {
    match name {
        SquashName::Commit(commit) => read_squash(git, commit).map_err(|error| squash_error_report(verb, &error)),
        SquashName::Change(change) => squash_by_change(git, verb, change),
    }
}

fn squash_by_change(git: &Git, verb: &str, change: &ChangeId) -> Result<Squash, Report> {
    let Ok(refs) = git.list_refs(SQUASH_REFS) else {
        return Err(failure_report(verb, "git could not list the squash refs under refs/mortar/squash/"));
    };
    let read: Vec<(String, Oid, Option<Squash>)> = refs
        .into_iter()
        .map(|(commit, refname)| {
            let squash = read_squash(git, &commit).ok();
            (refname, commit, squash)
        })
        .collect();
    let squashes: Vec<(String, Oid, Option<ChangeId>)> = read
        .iter()
        .map(|(refname, commit, squash)| {
            (refname.clone(), commit.clone(), squash.as_ref().map(|squash| squash.change_id.clone()))
        })
        .collect();
    if let Some(refusal) = squash_pick_report(verb, change, &pick_squash_ref(change, &squashes)) {
        return Err(refusal);
    }
    // No refusal means exactly one ref carries the change ID: its squash, already read.
    read.into_iter()
        .find_map(|(_, _, squash)| squash.filter(|squash| squash.change_id == *change))
        .ok_or_else(|| failure_report(verb, "the squash carrying the change ID was not among the refs read"))
}
