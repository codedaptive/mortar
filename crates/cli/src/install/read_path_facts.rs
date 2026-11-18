//! read_path_facts: Reads the PATH half of install's facts: the first `mr` and `mortar` on PATH, and where
//! `~/.local/bin` is on it. Reads files, never runs them.
//! No sibling test: it touches the disk; the install ZITs in install/integration/ cover it (placement, harness and
//! refusal: a fake `mr` earlier on PATH, a `mortar` under a fake prefix on PATH, `~/.local/bin` on PATH or not).

use std::fs;
use std::path::Path;

use crate::path::{file_has_marker, first_on_path, is_this_build, path_entries, MORTAR_MARKER};

use super::{dir_position, InstallEnv, InstallPaths, PathFacts};

/// The PATH facts for `env`: the first `mr` and `mortar` over `env.path` (empty and relative entries under
/// `env.cwd`), each as PATH names it with whether it is this Mortar build (is_this_build against `env.exe`; false
/// when the binary's path is unknown); the entry index the `mr` was found in; and the index of the first entry
/// equal to `paths.bin_dir`.
pub fn read_path_facts(env: &InstallEnv, paths: &InstallPaths) -> PathFacts {
    let path = env.path.as_deref();
    let entries = path_entries(path, &env.cwd);
    let mortar = |candidate: &Path| {
        env.exe.as_ref().is_some_and(|exe| {
            is_this_build(candidate, exe, &|p| fs::canonicalize(p).ok(), &|p| file_has_marker(p, &MORTAR_MARKER))
        })
    };
    let first = |name: &str| first_on_path(name, path, &env.cwd).map(|found| {
        let is = mortar(&found);
        (found, is)
    });
    let first_mr = first("mr");
    PathFacts {
        first_mr_at: first_mr.as_ref().and_then(|(found, _)| entries.iter().position(|e| e.join("mr") == *found)),
        first_mr,
        first_mortar: first("mortar"),
        bin_at: dir_position(&paths.bin_dir, &entries),
    }
}
