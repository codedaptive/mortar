//! kill_process_group: signal a run_in_group child's process group (Unix), and see its leader's exit without reaping
//! it (Linux and macOS).
//! The crate's only `unsafe` (S-38): std starts a process group (CommandExt::process_group) but cannot signal one, nor
//! see a child's exit without reaping it; libc or nix would be a new dependency for two calls. So this file declares
//! kill(2) and waitid(2) itself, and their two calls are the only unsafe code in it. S-38 puts both externs in this
//! one file, which is why it holds two functions, kill_process_group and peek_leader_exit, against one symbol per file.
//! No unit test: it is the extern calls only (S-5); the choices are group_kill_target's and peek_exit_call's, which
//! have their tests.
//! Non-Unix: the externs and the Unix bodies are compiled out (S-38); the two stubs there do nothing. No group is
//! started; kill_process_group sends nothing; peek_leader_exit gives None, so run_formatter_child falls back to
//! try_wait; the timeout kills only the child process.

use std::io;

#[cfg(unix)]
use crate::group_kill_target::group_kill_target;
#[cfg(unix)]
use crate::peek_exit_call::peek_exit_call;

/// SIGKILL: 9 on Linux and macOS.
#[cfg(unix)]
const SIGKILL: i32 = 9;
/// ESRCH, "no such process": 3 on Linux and macOS. For a group it means every member is already gone.
#[cfg(unix)]
const ESRCH: i32 = 3;

/// Room for waitid's siginfo_t: 256 bytes, more than Linux's 128 and macOS's 104, aligned for its pointer fields.
#[cfg(unix)]
#[repr(C, align(8))]
struct SigInfoBuf([i32; 64]);

// idtype_t and id_t are C unsigned ints, pid_t a C int, on Linux and macOS.
#[cfg(unix)]
extern "C" {
    fn kill(pid: i32, sig: i32) -> i32;
    fn waitid(idtype: u32, id: u32, infop: *mut SigInfoBuf, options: i32) -> i32;
}

/// Sends SIGKILL to every process in the group `leader_pid` leads. A pid group_kill_target refuses (0, 1, over
/// i32::MAX) sends nothing and is Ok; a group already gone (ESRCH) is Ok; any other kill error is the Err. Callers
/// must only call it while the leader is unreaped: once it is reaped its group id may belong to another process.
#[cfg(unix)]
pub(crate) fn kill_process_group(leader_pid: u32) -> io::Result<()> {
    let Some(target) = group_kill_target(leader_pid) else {
        return Ok(());
    };
    // SAFETY: kill takes two integers and reads or writes no memory of this process.
    if unsafe { kill(target, SIGKILL) } == 0 {
        return Ok(());
    }
    let error = io::Error::last_os_error();
    match error.raw_os_error() {
        Some(ESRCH) => Ok(()),
        _ => Err(error),
    }
}

/// Whether the child `leader_pid` has exited, seen without reaping it: Some(true) exited, Some(false) still running,
/// None when this OS has no peek (peek_exit_call). WNOWAIT leaves the child waitable, so Child::wait or try_wait
/// still reaps it afterwards. An interrupted call is retried; any other waitid error is the Err.
#[cfg(unix)]
pub(crate) fn peek_leader_exit(leader_pid: u32) -> io::Result<Option<bool>> {
    let Some(args) = peek_exit_call(std::env::consts::OS) else {
        return Ok(None);
    };
    let mut info = SigInfoBuf([0; 64]);
    loop {
        // SAFETY: `info` is a live, writable buffer larger than siginfo_t and aligned for it; waitid writes only there.
        if unsafe { waitid(args.idtype, leader_pid, &mut info, args.options) } == 0 {
            // si_signo is the first int on Linux and macOS: SIGCHLD when the child has exited. With WNOHANG and no
            // exit, POSIX gives 0 there, and the buffer was zeroed, so it reads 0 either way.
            return Ok(Some(info.0[0] != 0));
        }
        let error = io::Error::last_os_error();
        if error.kind() != io::ErrorKind::Interrupted {
            return Err(error);
        }
    }
}

/// Non-Unix: no group was started, so there is none to signal.
#[cfg(not(unix))]
pub(crate) fn kill_process_group(_leader_pid: u32) -> io::Result<()> {
    Ok(())
}

/// Non-Unix: no peek; run_formatter_child falls back to try_wait.
#[cfg(not(unix))]
pub(crate) fn peek_leader_exit(_leader_pid: u32) -> io::Result<Option<bool>> {
    Ok(None)
}
