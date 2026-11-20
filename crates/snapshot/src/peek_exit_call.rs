//! peek_exit_call: the waitid(2) arguments that see a child's exit without reaping it, per OS.

use crate::waitid_args::WaitidArgs;

/// P_PID: wait for the one child whose pid is given. 1 in Linux's linux/wait.h and in macOS's sys/wait.h.
const P_PID: u32 = 1;
/// WNOHANG: return at once when the child has not exited. 1 on Linux (linux/wait.h) and macOS (sys/wait.h).
const WNOHANG: i32 = 1;
/// WEXITED: report a child that has exited. 4 on Linux (linux/wait.h) and macOS (sys/wait.h).
const WEXITED: i32 = 4;
/// WNOWAIT: leave the child waitable, so it stays a zombie holding its pid and group id. 0x01000000 in Linux's
/// linux/wait.h (libc src/unix/linux_like/mod.rs).
const WNOWAIT_LINUX: i32 = 0x0100_0000;
/// WNOWAIT on macOS: 0x20 in sys/wait.h (libc src/unix/bsd/apple/mod.rs).
const WNOWAIT_MACOS: i32 = 0x20;

/// The arguments for waitid(P_PID, pid, &info, WEXITED | WNOHANG | WNOWAIT) on `os`, named as in
/// std::env::consts::OS. None on any OS other than "linux" and "macos": its constants are not checked here, so there
/// is no peek and run_formatter_child falls back to try_wait.
pub(crate) fn peek_exit_call(os: &str) -> Option<WaitidArgs> {
    let wnowait = match os {
        "linux" => WNOWAIT_LINUX,
        "macos" => WNOWAIT_MACOS,
        _ => return None,
    };
    Some(WaitidArgs { idtype: P_PID, options: WEXITED | WNOHANG | wnowait })
}
