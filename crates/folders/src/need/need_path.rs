//! The one clean path need adds, or the reason it is refused.
//! Pure; need_path_tests exercises it.

use crate::plan::{sparse_paths, PlanError};

/// `path` cleaned as sparse_paths cleans it, every refusal (a line break among them) passed through, so need
/// refuses before it writes anything.
pub(crate) fn need_path(path: &str) -> Result<String, PlanError> {
    Ok(sparse_paths(&[path.to_string()], &[])?.remove(0))
}
