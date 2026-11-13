//! merge_base_request: the git run that finds the best common ancestor of two commits.

use crate::{Oid, RunOpts};
use std::ffi::OsString;

/// `merge-base --end-of-options <a> <b>`: `--end-of-options` goes before caller revisions, as SPEC-gitio asks of
/// every run that takes them. Exit 1 (no common ancestor) is an answer; read_merge_base tells it from a failure by
/// its empty output.
pub(crate) fn merge_base_request(a: &Oid, b: &Oid) -> (Vec<OsString>, RunOpts) {
    (
        vec![
            "merge-base".into(),
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
