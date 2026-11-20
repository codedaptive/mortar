//! literal_exclude: a refused path as a pathspec that leaves out only that path.

use std::path::Path;

/// None when the path is not UTF-8 (excludes are passed as String) or empty (nothing to exclude). Else ":", a
/// backslash, then the path with each later `\`, `*`, `?` and `[` backslash-escaped (a first character that is one
/// of those gets the one leading backslash). gitio's snapshot_tree prefixes each exclude with `:!`, so git reads
/// `:!:\<escaped>`. The second `:` ends the short magic, so a path that starts with `:` stays whole. The leading
/// escape makes git match the whole pattern with fnmatch and no literal prefix, so it matches only that exact
/// path: without it git also matches a directory of that name, and a HEAD directory replaced by a refused file of
/// the same name would keep its files in the tree. The other escapes make glob characters literal. Checked
/// against git.
pub(crate) fn literal_exclude(path: &Path) -> Option<String> {
    let mut chars = path.to_str()?.chars();
    let first = chars.next()?;
    let mut exclude = String::from(":\\");
    exclude.push(first);
    for c in chars {
        if matches!(c, '\\' | '*' | '?' | '[') {
            exclude.push('\\');
        }
        exclude.push(c);
    }
    Some(exclude)
}
