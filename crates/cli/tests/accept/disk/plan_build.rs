//! Plans a zone folder's debug build from HEAD's tree.

use super::{BuildCommand, TreeFile};

/// `cargo build` in `dir` when a regular file (mode 100644 or 100755) sits at `<dir>/Cargo.toml` ("Cargo.toml" when
/// `dir` is "", the copy's top level); else `swift build` when one sits at `<dir>/Package.swift`. Both are debug
/// builds. Err naming `dir` (or the copy's top level) when it holds neither.
pub fn plan_build(files: &[TreeFile], dir: &str) -> Result<BuildCommand, String> {
    let holds = |name: &str| {
        let path = if dir.is_empty() { name.to_string() } else { format!("{dir}/{name}") };
        files.iter().any(|file| file.path == path && (file.mode == "100644" || file.mode == "100755"))
    };
    let program = if holds("Cargo.toml") {
        "cargo"
    } else if holds("Package.swift") {
        "swift"
    } else {
        let place = if dir.is_empty() { "the copy's top level" } else { dir };
        return Err(format!("{place} holds no Cargo.toml or Package.swift"));
    };
    Ok(BuildCommand { dir: dir.to_string(), program: program.to_string(), args: vec!["build".to_string()] })
}
