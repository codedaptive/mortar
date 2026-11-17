//! set_index_to_branch: an undo's SetIndex folder step, the folder's index set to its branch's tree (S-73, S-142).
//! No unit test (S-104): it runs git, snapshot or the op log; its decisions have their tests.

use crate::join::RequestRun;
use crate::report::{failure_report, Report};

const VERB: &str = "undo";

/// The folder's index set to the tree of the commit its branch holds; its files and HEAD untouched.
pub(super) fn set_index_to_branch(run: &RequestRun) -> Result<(), Report> {
    let failed = |_| failure_report(VERB, "git failed while setting the folder's index to its branch's tree");
    let Some(commit) = run.git.read_ref(&run.request_ref).map_err(failed)? else {
        return Err(failure_report(VERB, &format!("{} is gone, so the folder's index was not set", run.request_ref)));
    };
    let tree = run.git.commit_info(&commit).map_err(failed)?.tree;
    run.git.set_index(&run.folder.toplevel, &tree).map_err(failed)
}
