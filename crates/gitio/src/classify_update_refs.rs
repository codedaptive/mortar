//! classify_update_refs: read the result of `git update-ref --stdin -z` as an outcome.

use crate::{GitError, Output, RefUpdate, UpdateRefsOutcome};

/// The wordings git uses after `cannot lock ref '<refname>': ` when a ref's current value is not the expected one,
/// matched as prefixes. The first two were captured from git 2.56 (files and reftable backends); the third was not
/// reproduced there and is matched in case a git version words a missing ref that way. The missing-ref wording git
/// 2.56 does use is bound to the refname and checked by `is_missing_ref`.
const STALE_REASONS: [&str; 3] = ["is at ", "reference already exists", "reference is missing but expected"];

/// Whether `rest` is exactly `unable to resolve reference '<refname>'`, ending at a newline or the end of stderr.
/// Git 2.56 uses the same lead, followed by `: <cause>`, for a broken or unreadable ref; those are not a stale
/// value, so the wording is not matched as a prefix.
fn is_missing_ref(rest: &str, refname: &str) -> bool {
    rest.strip_prefix(format!("unable to resolve reference '{refname}'").as_str())
        .is_some_and(|tail| tail.is_empty() || tail.starts_with('\n'))
}

/// Ok(Updated) when git succeeded. When git answered with an exit code and its stderr says a sent ref was not at
/// its expected value, Ok(Stale) naming the first such ref in update order. Anything else is returned unchanged.
/// The refnames that were sent are matched whole, since a refname may contain `'`.
pub(crate) fn classify_update_refs(
    result: Result<Output, GitError>,
    updates: &[RefUpdate],
) -> Result<UpdateRefsOutcome, GitError> {
    let err = match result {
        Ok(_) => return Ok(UpdateRefsOutcome::Updated),
        Err(err) => err,
    };
    if err.code.is_none() {
        return Err(err);
    }
    let stale = updates.iter().find(|update| {
        let lead = format!("cannot lock ref '{}': ", update.refname);
        err.stderr.match_indices(&lead).any(|(at, _)| {
            let rest = &err.stderr[at + lead.len()..];
            STALE_REASONS.iter().any(|reason| rest.starts_with(reason)) || is_missing_ref(rest, &update.refname)
        })
    });
    match stale {
        Some(update) => Ok(UpdateRefsOutcome::Stale {
            refname: update.refname.clone(),
        }),
        None => Err(err),
    }
}
