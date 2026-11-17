//! Which folders `git worktree list` does not show as worktrees (check 15).

/// `porcelain` is `git worktree list --porcelain`: each worktree starts with a `worktree <path>` line. Returns, in
/// the order given, each of `folders` (absolute paths, as git prints them) that no such line names.
pub fn worktrees_missing(porcelain: &str, folders: &[String]) -> Vec<String> {
    let listed: Vec<&str> = porcelain.lines().filter_map(|line| line.strip_prefix("worktree ")).collect();
    folders.iter().filter(|folder| !listed.contains(&folder.as_str())).cloned().collect()
}
