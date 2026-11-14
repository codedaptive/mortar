//! run_formatter_child: run one formatter child, read its stderr, and kill and reap it if it runs past its limit.
//! No unit test: it is process I/O, and decide_formatter_wait, which decides when to stop waiting, has its tests
//! (S-5). This file gives TimedOut only when the formatter is still running: a reaped one cannot be killed, so its
//! own exit is the result.
//! The kill reaches the formatter process only, not processes it started. Once the formatter is reaped, the call
//! still waits, up to the limit, for its stderr to reach EOF. A process it started that keeps stderr open makes the
//! call wait the full limit; the result is then Exited with the formatter's own exit code and the stderr read so far
//! (not TimedOut), so the call stays bounded by the limit. In that case and on TimedOut the reader thread is not
//! joined; it ends when the pipe closes.
//! Nagatha's integration group should cover, in a throwaway repository: a formatter still running at a short timeout
//! (program `sh`, args ["-c", "sleep 30", "sh"]) is listed TimedOut with that limit, the snapshot is written, the call
//! returns long before 30 s, and no zombie is left; a formatter that writes to stderr and then hangs keeps that
//! stderr in TimedOut; a formatter that exits before its timeout gives Formatted or Failed; a formatter
//! writing more than FORMATTER_STDERR_CAP bytes of stderr is not blocked; a formatter that exits 0 after starting a
//! background process that holds stderr open is listed Formatted, after the limit.

use crate::child_exit::ChildExit;
use crate::decide_formatter_wait::decide_formatter_wait;
use crate::formatter_stderr_cap::FORMATTER_STDERR_CAP;
use crate::formatter_wait::FormatterWait;
use std::ffi::OsString;
use std::io::{self, Read};
use std::path::Path;
use std::process::{ChildStderr, Command, Stdio};
use std::sync::{Arc, Mutex, PoisonError};
use std::thread;
use std::time::{Duration, Instant};

/// How long the loop sleeps between polls. It bounds how late past the limit the kill comes; at 10 ms the polling
/// costs little CPU.
const POLL_INTERVAL: Duration = Duration::from_millis(10);

/// Runs `program` with `argv` in `folder`, stdin and stdout null. Gives Exited with its exit code (None when killed
/// by a signal) and the first FORMATTER_STDERR_CAP bytes of its stderr when it exits and its stderr reaches EOF; or,
/// when it has exited but its stderr is still open `limit` after its spawn, Exited with its exit code and the stderr
/// read so far; or, when it is still running `limit` after its spawn, TimedOut with `limit` and the stderr read so
/// far, after it is killed and reaped. A thread drains stderr meanwhile, so the child never blocks writing to a full
/// pipe. An error from spawning, polling or reaping is the Err, as is a stderr read error (or a panicked reader) where
/// the reader is joined, on the exited-and-EOF path; on the other two paths the reader is detached and a stderr read
/// error is dropped. The child is reaped before a polling error returns.
pub(crate) fn run_formatter_child(
    program: &Path,
    argv: &[OsString],
    folder: &Path,
    limit: Duration,
) -> io::Result<ChildExit> {
    let mut child = Command::new(program)
        .args(argv)
        .current_dir(folder)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()?;
    let start = Instant::now();
    let kept = Arc::new(Mutex::new(Vec::new()));
    let pipe = child.stderr.take();
    let reader = {
        let kept = Arc::clone(&kept);
        thread::spawn(move || pipe.map_or(Ok(()), |pipe| read_capped(pipe, &kept)))
    };
    let mut status = None;
    loop {
        if status.is_none() {
            status = child.try_wait().inspect_err(|_| {
                let _ = child.kill();
                let _ = child.wait();
            })?;
        }
        let finished = status.filter(|_| reader.is_finished());
        match (decide_formatter_wait(start.elapsed(), limit, finished.is_some()), finished) {
            (FormatterWait::Done, Some(exit)) => {
                reader.join().map_err(|_| io::Error::other("the formatter's stderr reader panicked"))??;
                return Ok(ChildExit::Exited { code: exit.code(), stderr: kept_bytes(&kept) });
            }
            (FormatterWait::Kill, _) => {
                return Ok(match status {
                    // Reaped already, with stderr held open by a process it started: its own exit is the result.
                    Some(exit) => ChildExit::Exited { code: exit.code(), stderr: kept_bytes(&kept) },
                    None => {
                        // It may have exited since try_wait; a failed kill then changes nothing, and wait reaps it.
                        let _ = child.kill();
                        child.wait()?;
                        ChildExit::TimedOut { limit, stderr: kept_bytes(&kept) }
                    }
                });
            }
            _ => thread::sleep(POLL_INTERVAL),
        }
    }
}

/// Reads `pipe` to EOF in chunks, keeping its first FORMATTER_STDERR_CAP bytes in `kept` and dropping the rest.
fn read_capped(mut pipe: ChildStderr, kept: &Mutex<Vec<u8>>) -> io::Result<()> {
    let mut chunk = [0u8; 8192];
    loop {
        let read = match pipe.read(&mut chunk) {
            Ok(0) => return Ok(()),
            Ok(read) => read,
            Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
            Err(e) => return Err(e),
        };
        let mut kept = kept.lock().unwrap_or_else(PoisonError::into_inner);
        let room = FORMATTER_STDERR_CAP.saturating_sub(kept.len());
        kept.extend_from_slice(&chunk[..read.min(room)]);
    }
}

/// A copy of the stderr kept so far. A poisoned lock still holds whole chunks, so its bytes are used as they are.
fn kept_bytes(kept: &Mutex<Vec<u8>>) -> Vec<u8> {
    kept.lock().unwrap_or_else(PoisonError::into_inner).clone()
}
