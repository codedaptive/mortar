//! Makes one step's throwaway repositories. Real git: the shell step (steps/shell.rs) checks what it made on every
//! drive.

use std::ffi::OsStr;
use std::fs;
use std::io;
use std::path::Path;
use std::process::{Command, Stdio};

use super::{world_bin_dir, world_env, Scratch, World};

impl World {
    /// In a fresh scratch folder: `bin` (world_bin_dir), first on `path`, the PATH of every git this runs;
    /// `work` (`git init -b main`) with Cargo.toml, src/a/a.txt and src/b/b.txt in one commit; `origin.git`, a bare
    /// repository pushed `main` from work and named `origin` there; `integration` branched from main in work;
    /// `other`, a clone of origin. `base` is that one commit. An error names the git command that failed and its
    /// stderr.
    pub fn new() -> io::Result<World> {
        let scratch = Scratch::new()?;
        let root = scratch.path().to_path_buf();
        let drive_path = std::env::var_os("PATH").unwrap_or_default();
        let bin = world_bin_dir(&root)?;
        let path = std::env::join_paths(std::iter::once(bin).chain(std::env::split_paths(&drive_path)))
            .map_err(io::Error::other)?;
        let (work, origin, other) = (root.join("work"), root.join("origin.git"), root.join("other"));
        let (work_text, origin_text, other_text) = (text(&work)?, text(&origin)?, text(&other)?);
        let git = |cwd: &Path, args: &[&str]| setup_git(&root, &path, cwd, args);
        git(&root, &["init", "-q", "-b", "main", work_text])?;
        fs::create_dir_all(work.join("src/a"))?;
        fs::create_dir_all(work.join("src/b"))?;
        fs::write(work.join("Cargo.toml"), "[workspace]\n")?;
        fs::write(work.join("src/a/a.txt"), "a\n")?;
        fs::write(work.join("src/b/b.txt"), "b\n")?;
        git(&work, &["add", "-A"])?;
        git(&work, &["commit", "-qm", "base"])?;
        git(&root, &["init", "-q", "--bare", "-b", "main", origin_text])?;
        git(&work, &["remote", "add", "origin", origin_text])?;
        git(&work, &["push", "-q", "origin", "main"])?;
        git(&work, &["branch", "integration", "main"])?;
        git(&work, &["fetch", "-q", "origin"])?;
        git(&root, &["clone", "-q", origin_text, other_text])?;
        let base = git(&work, &["rev-parse", "HEAD"])?.trim().to_string();
        Ok(World { root, work, origin, other, base, path, scratch })
    }
}

/// The scratch folder is made by the drive under the temp folder; a path that is not UTF-8 cannot be a git
/// argument here, so it is an error rather than a lossy copy.
fn text(path: &Path) -> io::Result<&str> {
    path.to_str().ok_or_else(|| io::Error::other(format!("not UTF-8: {}", path.display())))
}

/// Runs git with the world's environment and PATH and returns its stdout; a failure is an error naming the command.
fn setup_git(root: &Path, path: &OsStr, cwd: &Path, args: &[&str]) -> io::Result<String> {
    let output = Command::new("git")
        .env_clear()
        .envs(world_env(root, path))
        .current_dir(cwd)
        .args(args)
        .stdin(Stdio::null())
        .output()?;
    if !output.status.success() {
        return Err(io::Error::other(format!(
            "git {} in {}: {}: {}",
            args.join(" "),
            cwd.display(),
            output.status,
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}
