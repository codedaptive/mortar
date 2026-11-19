//! build.rs: embeds the commit the binary is built from as MORTAR_COMMIT, for `--version`: the first
//! 12 hex digits of HEAD, read through gitio, or "unknown" when git cannot read it (no repository, no git, an unborn
//! HEAD). So that the embedded commit follows HEAD, cargo reruns it when build.rs changes, when the git dir's HEAD
//! file changes, and, when HEAD names a branch, when that branch moves: its ref file (or, while the branch is only
//! packed, the folder its loose file would appear in) and packed-refs.
//! Only files that exist are named: cargo reruns a build script on every build while a named path is missing.

use std::path::{Path, PathBuf};

use gitio::Git;

const UNKNOWN: &str = "unknown";

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let git = Git::new(manifest);
    let commit = match git.head(manifest) {
        Ok((branch, oid)) => {
            watch(&git, manifest, branch.as_deref());
            oid.map_or_else(|| UNKNOWN.to_string(), |oid| oid.as_str().chars().take(12).collect())
        }
        Err(_) => UNKNOWN.to_string(),
    };
    println!("cargo:rustc-env=MORTAR_COMMIT={commit}");
}

/// Names the files whose change means HEAD moved; any that git cannot locate are left out.
fn watch(git: &Git, folder: &Path, branch: Option<&str>) {
    if let Ok(git_dir) = git.git_dir(folder) {
        rerun_if_exists(git_dir.join("HEAD"));
    }
    let (Some(branch), Ok(common)) = (branch, git.common_dir()) else {
        return;
    };
    let ref_file = common.join(branch);
    match ref_file.parent() {
        Some(parent) if !ref_file.exists() => rerun_if_exists(parent.to_path_buf()),
        _ => rerun_if_exists(ref_file),
    }
    rerun_if_exists(common.join("packed-refs"));
}

fn rerun_if_exists(path: PathBuf) {
    if path.exists() {
        println!("cargo:rerun-if-changed={}", path.display());
    }
}
