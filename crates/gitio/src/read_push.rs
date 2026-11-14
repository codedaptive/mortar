//! read_push: read the per-ref records of `push --porcelain` as one outcome per requested ref.

use crate::text_refusal::text_refusal;
use crate::{GitError, Output, PushOutcome, PushRef, PushRefusal};

/// One outcome per ref of `refs`, in that order, matched to git's records by refname (git's record order differs).
/// Stdout is lines ending in '\n': an optional first `To <url>` line and a `Done` line are skipped, every other line
/// is a record. Exit 1 with no records is a push git refused before sending any ref (a local pre-push hook, a
/// source it could not use): refused carrying git's stderr verbatim. Refused too: exit 0 with a refused record,
/// exit 1 with none, any other exit, a ref with no record or several, and a record for a ref nobody asked for.
pub(crate) fn read_push(output: &Output, refs: &[PushRef]) -> Result<Vec<PushOutcome>, GitError> {
    let records = records(&String::from_utf8_lossy(&output.stdout))?;
    let refused = records.iter().any(|(_, outcome)| matches!(outcome, PushOutcome::Refused(_)));
    match (output.status, records.is_empty(), refused) {
        (1, true, _) => {
            return Err(GitError::refusal(
                Vec::new(),
                format!("push exited 1 before sending any ref; git said:\n{}", String::from_utf8_lossy(&output.stderr)),
            ))
        }
        (0, _, false) | (1, _, true) => {}
        (status, _, refused) => {
            return Err(GitError::refusal(
                Vec::new(),
                format!("push exited {status} with a refused ref: {refused}"),
            ))
        }
    }
    if let Some((refname, _)) = records.iter().find(|(name, _)| !refs.iter().any(|r| r.refname == *name)) {
        return Err(text_refusal("push answered for a ref nobody asked for", refname));
    }
    refs.iter()
        .map(|push_ref| {
            let mut found = records.iter().filter(|(name, _)| *name == push_ref.refname);
            match (found.next(), found.next()) {
                (Some((_, outcome)), None) => Ok(outcome.clone()),
                _ => Err(text_refusal("push answered no single record for", &push_ref.refname)),
            }
        })
        .collect()
}

/// The records of `text`, skipping the `To ` line and `Done` (which git leaves out after some remote failures).
fn records(text: &str) -> Result<Vec<(String, PushOutcome)>, GitError> {
    if text.is_empty() {
        return Ok(Vec::new());
    }
    let body = text
        .strip_suffix('\n')
        .ok_or_else(|| text_refusal("push output does not end with a newline", text))?;
    let mut lines = body.split('\n').peekable();
    lines.next_if(|line| line.starts_with("To "));
    lines.filter(|line| *line != "Done").map(record).collect()
}

/// One record `<flag>\t<from>:<to>\t<summary>` as (to, outcome). `from` is an oid, empty for a delete, or
/// `(delete)` for a refused delete; none holds ':', so the first ':' ends it.
fn record(line: &str) -> Result<(String, PushOutcome), GitError> {
    let refuse = || text_refusal("not a push record", line);
    let mut fields = line.splitn(3, '\t');
    let (Some(flag), Some(from_to), Some(summary)) = (fields.next(), fields.next(), fields.next()) else {
        return Err(refuse());
    };
    let to = match from_to.split_once(':') {
        Some((_, to)) if !to.is_empty() => to,
        _ => return Err(refuse()),
    };
    let outcome = match flag {
        // ' ' fast-forward, '+' forced update, '-' deleted, '*' new ref.
        " " | "+" | "-" | "*" => PushOutcome::Pushed,
        "=" => PushOutcome::UpToDate,
        "!" => PushOutcome::Refused(refusal(summary)),
        _ => return Err(refuse()),
    };
    Ok((to.to_string(), outcome))
}

/// The reason in a refused record's summary, matched whole; any other summary is kept verbatim.
fn refusal(summary: &str) -> PushRefusal {
    match summary {
        "[rejected] (stale info)" => PushRefusal::StaleLease,
        "[rejected] (non-fast-forward)" => PushRefusal::NonFastForward,
        "[remote rejected] (pre-receive hook declined)" | "[remote rejected] (hook declined)" => PushRefusal::Hook,
        other => PushRefusal::Other(other.to_string()),
    }
}
