//! disk_usage_only_request: the git run that counts the disk bytes of objects only the given refs reach.

use crate::no_nul::no_nul;
use crate::text_refusal::text_refusal;
use crate::{GitError, RunOpts};
use std::ffi::OsString;

/// `rev-list --objects --disk-usage --not --exclude=<each ref> --all --not --end-of-options <each ref> --`,
/// read-only, refs in the order given. `--exclude` applies only to the next `--all`, so the excludes sit right before
/// it; the first `--not` makes `--all` negative and the second turns the given refs back to positive.
/// `--end-of-options` comes after them because git refuses `--not` past it; with it and the trailing `--`, git reports
/// an unknown ref as `fatal: bad revision '<ref>'`. Exit 128 is an answer, so read_disk_usage can tell an unknown ref
/// from git's other errors.
///
/// Each ref is checked, in order, before anything is built; the first that is not a usable full ref name is refused
/// naming it. Refused: empty; holding a NUL; not starting with `refs/`; holding `..` or `@{`; holding any of
/// `^ ~ : ? * [ \`. Why: rev-list reads a positive argument as a revision expression (`a..b` is a range, and
/// `^ ~ : @{` are revision syntax), and `--exclude=<pattern>` is a wildmatch glob against each ref's full name, so
/// `* ? [ \` would exclude other refs, while a name without them excludes exactly that full name. A short name
/// (`main`) would be excluded by nothing. git's check-ref-format forbids all of those characters and `..`/`@{` in a
/// ref name, so no real ref is refused; the `refs/` prefix also keeps a name from starting with `-`.
pub(crate) fn disk_usage_only_request<S: AsRef<str>>(refs: &[S]) -> Result<(Vec<OsString>, RunOpts), GitError> {
    for name in refs {
        usable(name.as_ref())?;
    }
    let mut args: Vec<OsString> = ["rev-list", "--objects", "--disk-usage", "--not"].map(OsString::from).to_vec();
    args.extend(refs.iter().map(|name| OsString::from(format!("--exclude={}", name.as_ref()))));
    args.extend(["--all", "--not", "--end-of-options"].map(OsString::from));
    args.extend(refs.iter().map(|name| OsString::from(name.as_ref())));
    args.push("--".into());
    Ok((
        args,
        RunOpts {
            answer_codes: vec![128],
            read_only: true,
            ..RunOpts::default()
        },
    ))
}

/// Ok for a full ref name git matches exactly as an `--exclude` pattern and as a revision; else a refusal naming it.
fn usable(name: &str) -> Result<(), GitError> {
    no_nul("ref name", name.as_bytes())?;
    let why = if name.is_empty() {
        "not a usable ref name (empty)"
    } else if !name.starts_with("refs/") {
        "not a full ref name (no refs/ prefix)"
    } else if name.contains("..") || name.contains("@{") {
        "ref name holds revision syntax (.. or @{)"
    } else if name.contains(['^', '~', ':', '?', '*', '[', '\\']) {
        "ref name holds a revision or glob character"
    } else {
        return Ok(());
    };
    Err(text_refusal(why, name))
}
