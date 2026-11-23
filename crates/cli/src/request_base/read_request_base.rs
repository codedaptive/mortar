//! read_request_base: the base and partners a request folder's record names, for a join, a resolve and status.
//! No unit test: it reads the record file from the folder's git dir. The integration tests
//! crates/cli/tests/save_join_runs/partner_join_tests.rs and crates/cli/tests/status_need_runs/status_partners_tests.rs
//! cover it through mr join, mr resolve and mr status in request folders cut with and without --with.

use folders::{read_recorded_base, RecordedBase};

use crate::context::FolderContext;
use crate::folder::folder_error_why;

/// folders' read_recorded_base on the folder's git dir. No record file is "the folder record of <toplevel> is
/// missing"; a FolderContext is made only from a record it read, so that answer means the file went away since. A
/// record that cannot be read is folder_error_why's lines joined with "; ".
pub(crate) fn read_request_base(folder: &FolderContext) -> Result<RecordedBase, String> {
    read_recorded_base(&folder.git_dir)
        .map_err(|error| folder_error_why(&error).join("; "))?
        .ok_or_else(|| format!("the folder record of {} is missing", folder.toplevel.display()))
}
