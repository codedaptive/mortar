//! rewrite_record_to: puts a request folder's record on a given base and partners before its folder moves, or reports
//! why it could not, for follow_answer (join's rebases and joined partner request), run_resolve (the rebase that
//! completes a paused join), finish_pending_move (a rewrite a kill left, or an undo's) and undo's folder_step (a
//! resolve step undone).
//! No unit test: it reads and writes the record file in the folder's git dir with git. The integration tests
//! crates/cli/tests/save_join_runs/{plain_base_tests,partner_join_tests}.rs, resolve_runs/resolve_base_tests.rs and
//! across_kits/undo_save_join_resolve/base_undo.rs cover it through mr join, mr resolve and mr undo in a throwaway
//! repository.

use folders::{rewrite_request_base, RecordedBase};
use gitio::Oid;

use super::RequestRun;
use crate::folder::folder_error_why;
use crate::report::{failure_report, with_moved_first, Report};
use crate::request_base::read_request_base;

/// Rewrites the folder record to the base `base` gives unless the record already holds exactly that base, so a
/// finished rewrite repeats as no write; Ok(None) writes nothing. A base that could not be read (its why), a failed
/// read of the record or a failed rewrite is failure_report for `verb`, with `moved` as its first line, saying the
/// record was not rewritten and the folder did not move: after `done` when given (what the caller's op moved), else
/// naming the folder and, when there is one, the base.
pub(crate) fn rewrite_record_to(
    run: &RequestRun,
    verb: &str,
    base: Result<Option<RecordedBase>, String>,
    done: Option<&str>,
    moved: Option<&str>,
) -> Result<(), Report> {
    let folder = &run.folder.toplevel;
    let not_rewritten = |onto: Option<&Oid>, why: String| {
        let what = match (done, onto) {
            (Some(done), _) => {
                format!("{done}; the folder record was not rewritten and the folder did not move: {why}")
            }
            (None, Some(onto)) => format!(
                "the folder record of {} was not rewritten to base {onto} and the folder did not move: {why}",
                folder.display()
            ),
            (None, None) => format!(
                "the folder record of {} was not rewritten and the folder did not move: {why}",
                folder.display()
            ),
        };
        let report = failure_report(verb, &what);
        Report { lines: with_moved_first(moved, report.lines), ..report }
    };
    let Some(base) = base.map_err(|why| not_rewritten(None, why))? else {
        return Ok(());
    };
    if read_request_base(run.folder).map_err(|why| not_rewritten(Some(&base.base), why))? == base {
        return Ok(());
    }
    rewrite_request_base(&run.git, folder, &base)
        .map_err(|error| not_rewritten(Some(&base.base), folder_error_why(&error).join("; ")))?;
    Ok(())
}
