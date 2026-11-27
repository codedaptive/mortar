//! read_need_path: the path `mr need` adds, cleaned and checked against the folder's commit before any lock.
//! No unit test: it runs git; need_error_report_tests and missing_path_report_tests cover the refusals
//! it gives. The integration tests crates/cli/tests/status_need_runs/need.rs cover, with the mortar binary as a child
//! process in a request folder made by the landing verbs in a throwaway repository (tests running real git set
//! GIT_CONFIG_GLOBAL=/dev/null and GIT_CONFIG_NOSYSTEM=1):
//! - a path not at the folder's commit is exit 3, and nothing changed (record, cone, refs and op log);
//! - a bad path ("../x", an absolute path) is exit 3;
//! - a folder whose HEAD has no commit: need is refused with repeat_head_refusal, exit 3, nothing changed.

use std::path::Path;

use folders::plan::sparse_paths;
use folders::FolderError;
use gitio::Git;

use super::{missing_path_report, need_error_report};
use crate::report::{failure_report, repeat_head_refusal, Report};

/// `raw` cleaned as folders' sparse_paths cleans a cone path (its trailing '/' removed); a path it refuses is
/// need_error_report of the plan's refusal. The path must be in the tree of the commit the folder's HEAD stands at,
/// as gitio's path_kind answers; a path it does not find is missing_path_report. A HEAD with no commit is
/// repeat_head_refusal; a git failure is failure_report. Reads only.
pub fn read_need_path(git: &Git, toplevel: &Path, raw: &str) -> Result<String, Report> {
    let fail = |what: String| failure_report("need", &what);
    let clean = sparse_paths(&[raw.to_string()], &[])
        .map_err(|error| need_error_report(&FolderError::Plan(error)))?
        .into_iter()
        .next()
        .ok_or_else(|| fail(format!("the path {raw:?} cleaned to no path")))?;
    let Some(commit) = git.head(toplevel).map_err(|error| fail(error.to_string()))?.1 else {
        let line = format!("HEAD of the folder {} has no commit", toplevel.display());
        return Err(repeat_head_refusal("need", vec![line]));
    };
    match git.path_kind(&commit, Path::new(&clean)).map_err(|error| fail(error.to_string()))? {
        Some(_) => Ok(clean),
        None => Err(missing_path_report(&clean, &commit)),
    }
}
