//! Which refs differ between two listings of `git for-each-ref --format=%(refname) %(objectname)`.

use std::collections::BTreeMap;

/// The names of the refs added, removed or moved from `before` to `after`, sorted. Each listing line is a ref name,
/// a space, its value; blank lines are left out.
pub fn refs_changed(before: &str, after: &str) -> Vec<String> {
    let (before, after) = (listing(before), listing(after));
    let mut names: Vec<String> = before.keys().chain(after.keys()).map(|name| name.to_string()).collect();
    names.sort();
    names.dedup();
    names.retain(|name| before.get(name.as_str()) != after.get(name.as_str()));
    names
}

fn listing(text: &str) -> BTreeMap<&str, &str> {
    text.lines().filter_map(|line| line.trim().split_once(' ')).collect()
}
