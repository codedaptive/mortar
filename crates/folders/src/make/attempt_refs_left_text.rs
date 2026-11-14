//! How an AttemptRefsLeft refusal reads: the moved ref and where it is, every attempt branch kept, the attempt
//! folders already removed (or that none was), and the next step.

use std::path::PathBuf;

use gitio::Oid;

/// The text of AttemptRefsLeft: `moved` and `at` (None reads "no commit"), then the folders at `removed`, joined
/// with ", ", or "no attempt folder was removed" when `removed` is empty.
pub(crate) fn attempt_refs_left_text(moved: &str, at: Option<&Oid>, removed: &[PathBuf]) -> String {
    let folders = if removed.is_empty() {
        "no attempt folder was removed".to_string()
    } else {
        format!(
            "the attempt folders {} were already removed",
            removed.iter().map(|p| p.display().to_string()).collect::<Vec<_>>().join(", ")
        )
    };
    format!(
        "ref {moved} moved (now at {}) before the attempt branches were deleted; every attempt branch is kept; \
         {folders}; next step: run the same command again with the same saved attempts",
        at.map_or("no commit", Oid::as_str)
    )
}
