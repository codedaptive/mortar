//! The cone set for a folder: the work order's paths and the paths it needs, as one clean list.

use super::PlanError;

/// The union of both lists, each path with its trailing '/' removed, sorted and deduplicated.
/// Refuses an empty union and, naming the path, an empty path, an absolute path, a path holding NUL,
/// and a path with a ".", ".." or empty component: each would reach outside the repository or name
/// no folder. Also refuses a path holding a line break ('\n'; a '\r' is kept): git writes it raw into the
/// sparse-checkout file, so gitio refuses it at sparse_set, after make has written branch, worktree and
/// record; refusing here keeps every refusal before any write.
pub fn sparse_paths(work_order_paths: &[String], needed_paths: &[String]) -> Result<Vec<String>, PlanError> {
    let mut cone = work_order_paths
        .iter()
        .chain(needed_paths)
        .map(|path| clean_path(path))
        .collect::<Result<Vec<String>, PlanError>>()?;
    if cone.is_empty() {
        return Err(PlanError {
            reason: "no sparse paths given: a folder needs at least one".to_string(),
        });
    }
    cone.sort();
    cone.dedup();
    Ok(cone)
}

fn clean_path(path: &str) -> Result<String, PlanError> {
    let refuse = |fault: &str| PlanError {
        reason: format!("sparse path {path:?} {fault}"),
    };
    if path.contains('\0') {
        return Err(refuse("holds NUL"));
    }
    if path.contains('\n') {
        return Err(refuse("holds a line break"));
    }
    if path.starts_with('/') {
        return Err(refuse("is absolute"));
    }
    let trimmed = path.trim_end_matches('/');
    if trimmed.is_empty() {
        return Err(refuse("is empty"));
    }
    if trimmed.split('/').any(|part| part.is_empty() || part == "." || part == "..") {
        return Err(refuse("has a \".\", \"..\" or empty component"));
    }
    Ok(trimmed.to_string())
}
