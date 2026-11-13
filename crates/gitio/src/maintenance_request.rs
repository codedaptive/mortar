//! maintenance_request: the git run that packs a repository's loose objects.

use crate::RunOpts;
use std::ffi::OsString;

/// `maintenance run --task=gc`, in the repo. Not read-only: it writes packs and removes loose objects. git skips the
/// run, exit 0 and silent, while another maintenance holds the repository's maintenance lock.
pub(crate) fn maintenance_request() -> (Vec<OsString>, RunOpts) {
    (["maintenance", "run", "--task=gc"].map(OsString::from).to_vec(), RunOpts::default())
}
