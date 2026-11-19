//! default_zone_start: the start a bare `mr folder zone` gives a missing zone branch.

/// `refs/heads/<origin_branch>` when no --start was given and the zone branch does not exist; None otherwise (a given
/// --start is used as it is, and an existing zone branch takes no start).
pub fn default_zone_start(start: Option<&str>, zone_branch_exists: bool, origin_branch: &str) -> Option<String> {
    (start.is_none() && !zone_branch_exists).then(|| format!("refs/heads/{origin_branch}"))
}
