//! commit_info_request: the git run that prints one commit's id, tree, parents and message.

use crate::{Oid, RunOpts};
use std::ffi::OsString;

/// `log -1 -z --no-show-signature --encoding=UTF-8 --format=%H%x00%T%x00%P%x00%B --end-of-options <commit> --`,
/// read-only. `--no-show-signature` keeps gpg lines out where log.showSignature is set; `--encoding=UTF-8` keeps
/// i18n.logOutputEncoding from re-encoding the message into bytes read_message would refuse. `--end-of-options`
/// goes before the revision and `--` after it, so the id is never read as an option or a path.
pub(crate) fn commit_info_request(commit: &Oid) -> (Vec<OsString>, RunOpts) {
    (
        vec![
            "log".into(),
            "-1".into(),
            "-z".into(),
            "--no-show-signature".into(),
            "--encoding=UTF-8".into(),
            "--format=%H%x00%T%x00%P%x00%B".into(),
            "--end-of-options".into(),
            commit.as_str().into(),
            "--".into(),
        ],
        RunOpts {
            read_only: true,
            ..RunOpts::default()
        },
    )
}
