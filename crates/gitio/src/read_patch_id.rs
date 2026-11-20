//! read_patch_id: read the answer of `patch-id`.

use crate::{GitError, Oid, PatchId};

/// Nothing on stdout is None: patch-id prints nothing for an empty diff. Otherwise stdout must be exactly one
/// `<patch id> <commit id>\n` line naming `commit`; anything else is refused naming what was found.
pub(crate) fn read_patch_id(stdout: &[u8], commit: &Oid) -> Result<Option<PatchId>, GitError> {
    if stdout.is_empty() {
        return Ok(None);
    }
    let fields = std::str::from_utf8(stdout)
        .ok()
        .and_then(|text| text.strip_suffix('\n'))
        .filter(|line| !line.contains('\n'))
        .and_then(|line| line.split_once(' '))
        .filter(|(_, named)| *named == commit.as_str());
    match fields.map(|(id, _)| PatchId::parse(id)) {
        Some(Ok(id)) => Ok(Some(id)),
        _ => Err(GitError::refusal(
            Vec::new(),
            format!(
                "patch-id answered {:?}, not one \"<patch id> {commit}\" line",
                String::from_utf8_lossy(stdout)
            ),
        )),
    }
}
