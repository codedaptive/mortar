//! Runs a program in the world under testkit's git kill shim. Real processes and a real SIGKILL: the shell step
//! (steps/shell.rs) kills a shell at an update-ref, before and after, on every drive.

use std::fs;
use std::io;
use std::path::Path;
use std::sync::atomic::{AtomicU32, Ordering};

use testkit::{run_with_kill_gate, write_git_kill_shim, write_git_shim};

use super::{Crash, Ran, World};

static NEXT: AtomicU32 = AtomicU32::new(0);

/// Puts the kill shim in front of real git. The wrapper holds no paths (they come by environment, so a quote in
/// one cannot break it), and its PATH is the real one, so the kill shim's own `git` is real git.
const WRAPPER: &[u8] = b"#!/bin/sh\nPATH=\"$ACCEPT_REAL_PATH\" exec \"$ACCEPT_KILL_SHIM\" \"$@\"\n";

/// `sh` names itself the worker, then execs the program: the exec keeps the pid, so the worker pid is the
/// program's and the parent of each git it starts, which the shim requires before it kills.
const WORKER: &str = "MORTAR_TEST_WORKER_PID=$$; export MORTAR_TEST_WORKER_PID; exec \"$@\"";

impl World {
    /// `program <args>` in `cwd`, killed by SIGKILL at the first `git -C <dir> update-ref` it runs, `crash`
    /// deciding whether that update-ref runs first; the child is reaped before this returns, with no timers. Its
    /// Ran shows signal 9 when it was killed, or how it ended when it never reached an update-ref.
    pub fn killed(&self, cwd: &Path, program: &Path, args: &[&str], crash: Crash) -> io::Result<Ran> {
        let dir = self.root.join(format!("kill-{}", NEXT.fetch_add(1, Ordering::Relaxed)));
        let bin = dir.join("bin");
        fs::create_dir_all(&bin)?;
        let shim = dir.join("git-kill.sh");
        write_git_kill_shim(&shim)?;
        write_git_shim(&bin.join("git"), WRAPPER)?;
        let real_path = std::env::var_os("PATH").unwrap_or_default();
        let path = std::env::join_paths(std::iter::once(bin).chain(std::env::split_paths(&real_path)))
            .map_err(io::Error::other)?;
        let mut command = self.command(Path::new("sh"), cwd);
        command
            .args(["-c", WORKER, "sh"])
            .arg(program)
            .args(args)
            .env("PATH", path)
            .env("ACCEPT_REAL_PATH", &real_path)
            .env("ACCEPT_KILL_SHIM", &shim)
            .env("MORTAR_TEST_CRASH", crash.as_str());
        run_with_kill_gate(&mut command, &dir).map(Ran::from_output)
    }
}
