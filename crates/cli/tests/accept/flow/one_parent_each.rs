//! Whether a list of commits is ordinary, one parent each (check 14).

/// `parents` is `git rev-list --parents <range>`: one line per commit, its id then its parents' ids. True when
/// there is at least one commit and every one has exactly one parent.
pub fn one_parent_each(parents: &str) -> bool {
    let mut lines = parents.lines().filter(|line| !line.trim().is_empty()).peekable();
    lines.peek().is_some() && lines.all(|line| line.split_whitespace().count() == 2)
}
