//! Runs the agent once. Real processes and files: run_agent.rs runs it on every drive with ACCEPT_AGENT set.

use std::ffi::{OsStr, OsString};
use std::fs::{self, File};
use std::io::{self, ErrorKind, Write};
use std::os::unix::process::ExitStatusExt;
use std::path::Path;
use std::process::{Command, ExitStatus, Stdio};

use super::AGENT_BRIEF;
use crate::world::{world_env, Ran, World};

/// `sh -c <command>` in `cwd` (the agent's request folder), AGENT_BRIEF on stdin, stdout and stderr both written
/// to `transcript` as they come; reaped before this returns, with no timer. Its environment is the drive's own,
/// so an agent tool finds its login and settings under the drive's HOME; over it the world's (world_env, its HOME
/// left out), then `env` (the agent's PATH and what the wrappers read); ACCEPT_DRIVE removed, so the wrappers log
/// its calls as the agent's. The Ran shows how it ended and holds no output (that is in the transcript); an agent
/// that could not start, or a transcript that could not be made, shows why.
pub fn agent_session(world: &World, cwd: &Path, env: &[(OsString, OsString)], command: &OsStr, transcript: &Path) -> Ran {
    match session(world, cwd, env, command, transcript) {
        Ok(status) => Ran { code: status.code(), signal: status.signal(), stdout: String::new(), stderr: String::new() },
        Err(error) => Ran::from_error("the agent", error),
    }
}

fn session(
    world: &World,
    cwd: &Path,
    env: &[(OsString, OsString)],
    command: &OsStr,
    transcript: &Path,
) -> io::Result<ExitStatus> {
    if let Some(folder) = transcript.parent() {
        fs::create_dir_all(folder)?;
    }
    let out = File::create(transcript)?;
    let mut child = Command::new("sh")
        .arg("-c")
        .arg(command)
        .current_dir(cwd)
        .envs(world_env(&world.root, &world.path).into_iter().filter(|(key, _)| key != "HOME"))
        .envs(env.iter().cloned())
        .env_remove("ACCEPT_DRIVE")
        .stdin(Stdio::piped())
        .stderr(out.try_clone()?)
        .stdout(out)
        .spawn()?;
    // The brief fits in a pipe's buffer, so the write never waits on the agent; an agent that ends without reading
    // it closes the pipe, which is its own outcome and not the drive's error.
    let fed = child.stdin.take().map_or(Ok(()), |mut stdin| stdin.write_all(AGENT_BRIEF.as_bytes()));
    let status = child.wait()?;
    match fed {
        Err(error) if error.kind() != ErrorKind::BrokenPipe => Err(error),
        _ => Ok(status),
    }
}
