//! snapshot_excludes: the paths a snapshot leaves out.

/// Build output kept out of every snapshot, as git pathspecs (gitio's snapshot_tree prefixes each with `:!`).
/// "<name>/" matches only the top-level directory and "*/<name>/*" the nested ones (checked against git), so each
/// name needs both. .gitignore needs no entry here because `git add -A` honours it.
const BUILT_IN: [&str; 6] = ["target/", "*/target/*", "node_modules/", "*/node_modules/*", ".build/", "*/.build/*"];

/// The built-in excludes in order, then each of `extra` in order, skipping one already in the list.
pub(crate) fn snapshot_excludes(extra: &[String]) -> Vec<String> {
    let mut excludes: Vec<String> = BUILT_IN.iter().map(|s| s.to_string()).collect();
    for path in extra {
        if !excludes.contains(path) {
            excludes.push(path.clone());
        }
    }
    excludes
}
