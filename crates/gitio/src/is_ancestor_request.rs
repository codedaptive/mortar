//! is_ancestor_request: the git run that asks whether one commit is an ancestor of another.

use crate::{Oid, RunOpts};
use std::ffi::OsString;

/// `merge-base --is-ancestor --end-of-options <a> <b>`, read-only: exit 0 when `a` is an ancestor of `b` (or is
/// `b`), exit 1 when it is not. Exit 1 is an answer, so it is an answer code; read_is_ancestor tells it from a
/// failure by its empty output. `--end-of-options` goes before the revisions, so neither is read as an
/// option.
pub(crate) fn is_ancestor_request(a: &Oid, b: &Oid) -> (Vec<OsString>, RunOpts) {
    (
        vec![
            "merge-base".into(),
            "--is-ancestor".into(),
            "--end-of-options".into(),
            a.as_str().into(),
            b.as_str().into(),
        ],
        RunOpts {
            answer_codes: vec![1],
            read_only: true,
            ..RunOpts::default()
        },
    )
}
