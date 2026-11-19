//! Actual child reaping before releasing a git crash wrapper; tests/kill_gate.rs covers the join.

use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::os::unix::process::ExitStatusExt;
use std::path::Path;
use std::process::{Command, Output, Stdio};

/// Runs a test child using write_git_kill_shim. A private FIFO keeps its git wrapper alive
/// after sending SIGKILL, until wait() has actually reaped the worker. No timers or polling.
/// The worker must pipe the git wrapper's output (as Git::run does), not inherit this child's
/// pipes. Unexpected normal/error exits return their real output for the caller to diagnose.
pub fn run_with_kill_gate(command: &mut Command, parent: &Path) -> io::Result<Output> {
    let directory = tempfile::Builder::new().prefix("kill-gate-").tempdir_in(parent)?;
    let release = directory.path().join("release");
    let ack = directory.path().join("ack");
    let armed = directory.path().join("armed");
    let made = Command::new("mkfifo").args(["-m", "600"]).arg(&release).arg(&ack).output()?;
    if !made.status.success() {
        return Err(io::Error::other(format!("mkfifo: {}: {}", made.status, String::from_utf8_lossy(&made.stderr))));
    }
    let mut child = command
        .env("TESTKIT_GATE_RELEASE", &release)
        .env("TESTKIT_GATE_ACK", &ack)
        .env("TESTKIT_GATE_ARMED", &armed)
        .stdout(Stdio::piped()).stderr(Stdio::piped()).spawn()?;
    let stdout = child.stdout.take();
    let stderr = child.stderr.take();
    std::thread::scope(|scope| {
        // Drain concurrently, so waiting for termination does not depend on pipe capacity.
        let stdout_reader = scope.spawn(move || drain(stdout));
        let stderr_reader = scope.spawn(move || drain(stderr));
        let waited = child.wait();
        if waited.is_err() {
            // Best-effort cleanup of our own child before releasing any armed wrapper.
            let _ = child.kill();
            let _ = child.wait();
        }
        let gate = if waited.as_ref().is_ok_and(|status| status.signal() == Some(9)) || waited.is_err() {
            release_gate(&armed, &release, &ack)
        } else {
            // A child that failed before reaching the wrapper has no gate reader to release.
            Ok(())
        };
        let stdout = stdout_reader.join().map_err(|_| io::Error::other("stdout reader panicked"))??;
        let stderr = stderr_reader.join().map_err(|_| io::Error::other("stderr reader panicked"))??;
        let status = waited?;
        if let Err(error) = gate {
            return Err(io::Error::other(format!("kill gate: {error}; child stdout: {}; stderr: {}",
                String::from_utf8_lossy(&stdout), String::from_utf8_lossy(&stderr))));
        }
        Ok(Output { status, stdout, stderr })
    })
}

fn drain<R: Read>(pipe: Option<R>) -> io::Result<Vec<u8>> {
    let mut pipe = pipe.ok_or_else(|| io::Error::other("child output was not piped"))?;
    let mut bytes = Vec::new();
    pipe.read_to_end(&mut bytes)?;
    Ok(bytes)
}

fn release_gate(armed: &Path, release: &Path, ack: &Path) -> io::Result<()> {
    // Refuse an unarmed boundary before any FIFO operation that could wait for a reader.
    match fs::read(armed) {
        Ok(marker) if marker == b"armed\n" => {}
        Ok(_) => return Err(io::Error::other("worker died without a valid armed crash boundary")),
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            return Err(io::Error::other("worker died without a valid armed crash boundary"));
        }
        Err(error) => return Err(error),
    }
    // Opening the release FIFO for writing waits for the wrapper's read side.
    let mut writer = OpenOptions::new().write(true).open(release)?;
    // Only the outer parent calls this, and only after waiting for worker termination.
    writer.write_all(b"release\n")?;
    drop(writer);
    let mut acknowledged = Vec::new();
    File::open(ack)?.read_to_end(&mut acknowledged)?;
    if acknowledged != b"reaped\n" {
        return Err(io::Error::other(format!("worker was not reaped before gate release: ack {:?}",
            String::from_utf8_lossy(&acknowledged))));
    }
    Ok(())
}
