//! commit_info_request: the git run that prints one commit's id, tree, parents, author and message.

use crate::{Oid, RunOpts};
use std::ffi::OsString;

/// `log -1 -z --no-show-signature --encoding=UTF-8 --date=raw --format=%H%x00%T%x00%P%x00%an%x00%ae%x00%ad%x00%B
/// --end-of-options <commit> --`, read-only. `--no-show-signature` keeps gpg lines out where log.showSignature is
/// set; `--encoding=UTF-8` keeps i18n.logOutputEncoding from re-encoding the message into bytes read_message would
/// refuse. `--date=raw` makes `%ad` print the author line's stored `<seconds> <offset>` verbatim, which
/// GIT_AUTHOR_DATE takes back unchanged; an author line with no date prints an empty `%ad`, which GIT_AUTHOR_DATE
/// cannot hand back (S-43).
/// `%an`/`%ae`, not `%aN`/`%aE`, so .mailmap never rewrites the identity.
/// `--end-of-options` goes before the revision and `--` after it, so the id is never read as an option or a path.
pub(crate) fn commit_info_request(commit: &Oid) -> (Vec<OsString>, RunOpts) {
    (
        vec![
            "log".into(),
            "-1".into(),
            "-z".into(),
            "--no-show-signature".into(),
            "--encoding=UTF-8".into(),
            "--date=raw".into(),
            "--format=%H%x00%T%x00%P%x00%an%x00%ae%x00%ad%x00%B".into(),
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
