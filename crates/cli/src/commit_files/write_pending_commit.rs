//! write_pending_commit: the pending marker of an `mr commit-files` run in a zone folder written before commit_paths.
//! No sibling test: it writes a file; pending_commit_text_tests covers the marker's text. The integration test
//! commit_files_zone_repeat_after_a_stopped_run_commits_once in crates/cli/tests/folder_runs/run_commit_files_tests.rs
//! covers the marker through the binary, and the other zone folder cases there run through this write.

use std::fs;
use std::path::Path;

use super::{pending_commit_text, PendingCommit};
use crate::report::{failure_report, Report};

/// pending_commit_text of `pending` written to `marker`. A path whose display holds a newline (it would not read
/// back as it was), or a write that fails, is failure_report naming the marker, with no commit written and no ref
/// moved.
pub fn write_pending_commit(marker: &Path, pending: &PendingCommit) -> Result<(), Report> {
    let failed = |why: &str| {
        let what = format!("the pending marker {} {why}; no commit was written and no ref moved", marker.display());
        failure_report("commit-files", &what)
    };
    if pending.paths.iter().any(|path| path.display().to_string().contains('\n')) {
        return Err(failed("cannot name a path holding a newline"));
    }
    fs::write(marker, pending_commit_text(pending)).map_err(|_| failed("could not be written"))
}
