//! run_formatter_child: run one formatter child as the leader of its own process group, read its stderr, and kill the
//! group, and the formatter if it is still running, at its end or its limit.
//! No unit test: it is process I/O, and decide_formatter_wait, decide_group_kill and decide_formatter_end, which
//! decide each poll, have their tests (S-5); kill_process_group holds the system calls.
//! On Unix the formatter leads a new group whose id is its pid. The group is signalled only while the formatter is
//! unreaped, since a reaped leader's group id may be reused (S-38). At its limit a running formatter's group is
//! killed, then the formatter is killed and reaped, and the call returns without waiting for the stderr read
//! (TimedOut). On Linux and macOS a normal exit is seen with WNOWAIT, the group is killed, then the formatter is
//! reaped: no process it started is left to write into the snapshot, and none holding stderr holds the call. On other
//! Unix there is no peek: after a normal exit (seen by try_wait, which reaps) the group is not signalled, so processes
//! it started are left, and one holding stderr holds the call until the limit (then Exited with the formatter's own
//! exit, the pipe held). Non-Unix: no group; the timeout kills the formatter only. A process that leaves the group
//! (setsid, setpgid) is out of reach of a group kill; the limit still bounds the call.
//! A stderr read error or a panicked reader is the Err only where the reader is joined (exited, stderr at EOF); where
//! it is left detached (TimedOut, or a held pipe at the limit) the read error is dropped.
//! Nagatha's integration tests cover it through snapshot_formatted, in a throwaway repository:
//! - a formatter blocked on a FIFO (`exec cat`) at a short limit is listed TimedOut with that limit and its stderr,
//!   its pid is gone and reaped, and the snapshot is written:
//!   tests/snapshot/formatter_timeout.rs::fifo_blocked_formatter_is_timed_out_with_limit_and_stderr_killed_reaped_and_snapshot_logged;
//! - a formatter that exits before its limit is listed Formatted and is reaped:
//!   formatter_timeout.rs::normal_formatter_exits_within_its_limit_and_is_reaped_without_timeout_result;
//! - one writing more than FORMATTER_STDERR_CAP bytes of stderr is drained, and is listed Failed with code 1 and the
//!   first 4096 bytes: formatter_timeout.rs::stderr_larger_than_pipe_and_cap_is_drained_but_only_first_4096_bytes_are_reported.
//! In tests/snapshot/formatter_group.rs (S-37) each case also shows that the background child's process group id is
//! the formatter's pid, the formatter is gone and reaped, the child is gone or a zombie, and the snapshot is written:
//! - an `sh` wrapper whose background child blocks is listed TimedOut with the limit and the wrapper's stderr:
//!   formatter_group.rs::wrapper_and_its_fifo_blocked_child_are_terminated_on_timeout_and_snapshot_is_written;
//! - on Linux and macOS, a formatter that exits 0 leaving a background child holding stderr is listed Formatted, not
//!   TimedOut, at a 60 s limit, and the group kill ends the child without the test releasing it:
//!   formatter_group.rs::normally_exited_leader_kills_child_holding_stderr_without_external_release_or_timeout_result;
//! - on Linux and macOS, a background child told to write a file after the formatter exits never writes it: the file
//!   on disk and in the snapshot tree keep their earlier contents:
//!   formatter_group.rs::background_write_released_after_leader_exit_cannot_enter_snapshot_tree.

use crate::child_exit::ChildExit;
use crate::decide_formatter_end::decide_formatter_end;
use crate::decide_formatter_wait::decide_formatter_wait;
use crate::decide_group_kill::decide_group_kill;
use crate::formatter_end::FormatterEnd;
use crate::kept_bytes::kept_bytes;
use crate::kill_process_group::{kill_process_group, peek_leader_exit};
use crate::leader_state::LeaderState;
use crate::read_capped::read_capped;
use std::ffi::OsString;
use std::io;
use std::path::Path;
use std::process::{Child, Command, ExitStatus, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

/// How long the loop sleeps between polls. It bounds how late past the limit the kill comes; at 10 ms the polling
/// costs little CPU.
const POLL_INTERVAL: Duration = Duration::from_millis(10);

/// Runs `program` with `argv` in `folder`, stdin and stdout null. Gives Exited with its exit code (None when killed
/// by a signal) and the first FORMATTER_STDERR_CAP bytes of its stderr when it exits and its stderr reaches EOF; or,
/// when it has exited but its stderr is still open `limit` after its spawn, Exited with its exit code and the stderr
/// read so far; or, when it is still running `limit` after its spawn, TimedOut with `limit` and the stderr read so
/// far, after its group and it are killed and it is reaped. A thread drains stderr meanwhile, so the child never
/// blocks writing to a full pipe. An error from spawning, polling or reaping is the Err, as is a stderr read error (or
/// a panicked reader) on the exited-and-EOF path. The child is reaped before a polling error returns.
pub(crate) fn run_formatter_child(
    program: &Path,
    argv: &[OsString],
    folder: &Path,
    limit: Duration,
) -> io::Result<ChildExit> {
    let mut command = Command::new(program);
    command.args(argv).current_dir(folder).stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::piped());
    #[cfg(unix)]
    std::os::unix::process::CommandExt::process_group(&mut command, 0);
    let mut child = command.spawn()?;
    let pid = child.id();
    let start = Instant::now();
    let kept = Arc::new(Mutex::new(Vec::new()));
    let pipe = child.stderr.take();
    let reader = {
        let kept = Arc::clone(&kept);
        thread::spawn(move || pipe.map_or(Ok(()), |pipe| read_capped(pipe, &kept)))
    };
    let mut status = None;
    let mut group_killed = false;
    loop {
        let leader = observe_leader(&mut child, &mut status).inspect_err(|_| {
            // No group signal: a failed wait (such as ECHILD) is no proof the leader is unreaped (S-38).
            let _ = child.kill();
            let _ = child.wait();
        })?;
        let step = decide_formatter_wait(start.elapsed(), limit, leader == LeaderState::Reaped && reader.is_finished());
        if decide_group_kill(leader, group_killed, step) {
            // Errors dropped: a group already gone (ESRCH) is Ok already, any other (such as EPERM) cannot be acted
            // on, and at the limit the formatter itself is still killed below.
            let _ = kill_process_group(pid);
            group_killed = true;
        }
        if leader == LeaderState::ExitedUnreaped {
            status = Some(child.wait()?);
        }
        match (decide_formatter_end(step, status.is_some()), status) {
            (None, _) => thread::sleep(POLL_INTERVAL),
            (Some(FormatterEnd::Exited), Some(exit)) => {
                reader.join().map_err(|_| io::Error::other("the formatter's stderr reader panicked"))??;
                return Ok(ChildExit::Exited { code: exit.code(), stderr: kept_bytes(&kept) });
            }
            (Some(FormatterEnd::ExitedPipeHeld), Some(exit)) => {
                return Ok(ChildExit::Exited { code: exit.code(), stderr: kept_bytes(&kept) });
            }
            // TimedOut: the one end reached with the formatter unreaped. It may already be dead from the group kill;
            // a failed kill then changes nothing, and wait reaps it.
            (Some(_), _) => {
                let _ = child.kill();
                child.wait()?;
                return Ok(ChildExit::TimedOut { limit, stderr: kept_bytes(&kept) });
            }
        }
    }
}

/// Where the formatter is, with `status` holding its exit once reaped. On Linux and macOS its exit is seen without
/// reaping it (peek_leader_exit); elsewhere try_wait sees it and reaps it.
fn observe_leader(child: &mut Child, status: &mut Option<ExitStatus>) -> io::Result<LeaderState> {
    if status.is_some() {
        return Ok(LeaderState::Reaped);
    }
    match peek_leader_exit(child.id())? {
        Some(true) => Ok(LeaderState::ExitedUnreaped),
        Some(false) => Ok(LeaderState::Running),
        None => {
            *status = child.try_wait()?;
            Ok(if status.is_some() { LeaderState::Reaped } else { LeaderState::Running })
        }
    }
}
