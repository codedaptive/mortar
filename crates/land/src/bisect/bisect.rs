//! bisect: name the first failing landing between a passing commit and a failed one on origin (07-bisect.md).
//! No unit test: it runs git and probes (S-5). Its refusals before any probe are decided by decide_bisect_range
//! (decide_bisect_range_tests) and the first-parent check by on_first_parent_line (on_first_parent_line_tests).
//! tests/bisect_zit.rs covers the run: a merge landing named through its squash's trailers
//! (bisect_names_the_merge_landing_that_broke_the_script), a timed-out probe as a fail
//! (a_timed_out_probe_counts_as_a_fail), a probe that cannot start (a_command_that_cannot_start_stops_the_search),
//! last_pass off the line (a_last_pass_off_the_first_parent_line_is_refused), a failed commit off origin
//! (a_failed_commit_off_origin_is_refused) and a landing without trailers
//! (a_landing_without_trailers_is_still_named).

use std::time::Duration;

use gitio::{Git, Oid};
use oplog::{read_mortar_trailers, MortarTrailerError};

use crate::LandRefs;

use super::probe_commit::probe_commit;
use super::{
    culprit_of_landing, decide_bisect_range, on_first_parent_line, search_first_failing, BisectAnswer, BisectError,
    BisectReport, BisectVerdict, Culprit, ProbeOutcome, TestCommand,
};

/// The first commit after `last_pass` up to `failed` on origin's first-parent line that fails `test_command`, with
/// every probe in the order run.
///
/// Refused before any probe: OriginMissing when the origin branch is absent, NotOnOrigin when `failed` is not on
/// its first-parent line, OffTheLine when `last_pass` is not on `failed`'s (S-78). Each probe runs the command in
/// its own throwaway detached worktree with `timeout` as its limit: exit 0 passes, any other end fails, and a
/// command that could not be run stops the search with ProbeCouldNotRun. No git bisect state is written and no ref
/// moves.
pub fn bisect(
    git: &Git,
    refs: &LandRefs,
    last_pass: &Oid,
    failed: &Oid,
    test_command: &TestCommand,
    timeout: Duration,
) -> Result<BisectReport, BisectError> {
    let origin_ref = format!("refs/heads/{}", refs.origin_branch);
    let tip = git
        .read_ref(&origin_ref)
        .map_err(BisectError::Git)?
        .ok_or(BisectError::OriginMissing { refname: origin_ref })?;
    let candidates = decide_bisect_range(refs, &tip, last_pass, failed, |base, tip| first_parent_line(git, base, tip))?;
    let mut probes = Vec::new();
    let verdict = search_first_failing(last_pass, &candidates, |commit| {
        let probe = probe_commit(git, commit, test_command, timeout)?;
        let passed = probe.outcome == ProbeOutcome::Passed;
        probes.push(probe);
        Ok(passed)
    })?;
    let answer = match verdict {
        BisectVerdict::FirstFailing(commit) => BisectAnswer::Culprit(culprit_at(git, &commit)?),
        BisectVerdict::NotFound(why) => BisectAnswer::NotFound(why),
    };
    Ok(BisectReport { answer, probes })
}

/// `tip`'s first-parent commits that `base` cannot reach, oldest first, when `base` is on that line; else None.
fn first_parent_line(git: &Git, base: &Oid, tip: &Oid) -> Result<Option<Vec<Oid>>, BisectError> {
    let line = git.first_parent_list(base, tip).map_err(BisectError::Git)?;
    let oldest_first_parent = match line.first() {
        Some(oldest) => git.commit_info(oldest).map_err(BisectError::Git)?.parents.into_iter().next(),
        None => None,
    };
    Ok(on_first_parent_line(base, tip, &line, oldest_first_parent.as_ref()).then_some(line))
}

/// The Culprit for landing `commit`, read from its squash's Mortar trailers.
fn culprit_at(git: &Git, commit: &Oid) -> Result<Culprit, BisectError> {
    let parents = git.commit_info(commit).map_err(BisectError::Git)?.parents;
    culprit_of_landing(commit, &parents, |squash| {
        read_mortar_trailers(git, &git.commit_info(squash).map_err(MortarTrailerError::Git)?.message)
    })
    .map_err(BisectError::Culprit)
}
