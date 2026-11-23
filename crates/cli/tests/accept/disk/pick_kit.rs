//! Picks the one kit step 5 makes a zone of, from HEAD's tree.

use std::collections::BTreeSet;

use super::TreeFile;

/// The kit rule. A package folder is a folder below the top level that directly holds a regular file (mode 100644
/// or 100755) named Cargo.toml or Package.swift. Of the package folders that also hold a regular file at least one
/// folder below them, the kit is the one whose entries' sizes (None counts 0) sum lowest, a tie going to the name
/// that sorts first. Err when no folder qualifies.
pub fn pick_kit(files: &[TreeFile]) -> Result<String, String> {
    let regular = |file: &TreeFile| file.mode == "100644" || file.mode == "100755";
    let packages: BTreeSet<&str> = files
        .iter()
        .filter(|file| regular(file))
        .filter_map(|file| file.path.rsplit_once('/'))
        .filter(|(_, name)| matches!(*name, "Cargo.toml" | "Package.swift"))
        .map(|(dir, _)| dir)
        .collect();
    packages
        .into_iter()
        .filter_map(|dir| {
            let prefix = format!("{dir}/");
            let inside: Vec<(&TreeFile, &str)> =
                files.iter().filter_map(|file| file.path.strip_prefix(&prefix).map(|rest| (file, rest))).collect();
            let deep = inside.iter().any(|(file, rest)| regular(file) && rest.contains('/'));
            // (bytes, name) tuples order by bytes, then by name: min is the lightest, a tie the first name.
            deep.then(|| (inside.iter().map(|(file, _)| file.size.unwrap_or(0)).sum::<u64>(), dir))
        })
        .min()
        .map(|(_, dir)| dir.to_string())
        .ok_or_else(|| {
            "HEAD holds no package folder (a folder below the top level with its own Cargo.toml or Package.swift and a \
             file in a subfolder)"
                .to_string()
        })
}
