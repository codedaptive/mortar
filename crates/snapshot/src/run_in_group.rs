//! run_in_group: run one command (on Unix as the leader of its own process group), and say how it ended.
//! No unit test: it is process I/O and holds no decision of its own; decide_group_run_end, decide_formatter_wait,
//! decide_group_kill and decide_formatter_end have their tests.
//! The integration tests in tests/snapshot cover it through snapshot_formatted, in a throwaway repository:
//! - a missing program is SpawnFailed with the OS's error text ("No such file or directory"), snapshot written:
//!   tests/snapshot/formatter.rs::missing_formatter_is_spawn_failed_and_does_not_prevent_snapshot_or_log;
//! - a formatter blocked on a FIFO (`exec cat`) at a short limit is listed TimedOut with that limit and its stderr,
//!   its pid is gone and reaped, and the snapshot is written:
//!   tests/snapshot/formatter_timeout.rs::fifo_blocked_formatter_is_timed_out_with_limit_and_stderr_killed_reaped_and_snapshot_logged;
//! - a formatter that exits before its limit is listed Formatted and is reaped:
//!   formatter_timeout.rs::normal_formatter_exits_within_its_limit_and_is_reaped_without_timeout_result;
//! - one writing more than FORMATTER_STDERR_CAP bytes of stderr is drained, and is listed Failed with code 1 and the
//!   first 4096 bytes: formatter_timeout.rs::stderr_larger_than_pipe_and_cap_is_drained_but_only_first_4096_bytes_are_reported.
//!
//! In tests/snapshot/formatter_group.rs each case also shows that the background child's process group id is
//! the formatter's pid, the formatter is gone and reaped, the child ended without being released, and the snapshot is written:
//! - an `sh` wrapper whose background child blocks is listed TimedOut with the limit and the wrapper's stderr:
//!   formatter_group.rs::wrapper_and_its_fifo_blocked_child_are_terminated_on_timeout_and_snapshot_is_written;
//! - on Linux and macOS, a formatter that exits 0 leaving a background child holding stderr is listed Formatted, not
//!   TimedOut, at a 60 s limit, and the group kill ends the child without the test releasing it:
//!   formatter_group.rs::normally_exited_leader_kills_child_holding_stderr_without_external_release_or_timeout_result;
//! - on Linux and macOS, a background child told to write a file after the formatter exits never writes it: the file
//!   on disk and in the snapshot tree keep their earlier contents:
//!   formatter_group.rs::background_write_released_after_leader_exit_cannot_enter_snapshot_tree.

use crate::decide_group_run_end::decide_group_run_end;
use crate::group_run_end::GroupRunEnd;
use crate::run_formatter_child::run_formatter_child;
use std::ffi::OsString;
use std::path::Path;
use std::time::Duration;

/// Runs the command `program` then `args` in `folder` (its current directory), with no shell, stdin and stdout null,
/// stderr piped and drained, and the environment inherited. On Unix it leads its own process group, and the group is
/// killed and the command reaped at its end or its `limit` as run_formatter_child's header says; on
/// other systems there is no group, and the limit kills the command only. It never returns an error: every io error
/// is SpawnFailed. Its caller is run_formatter; it is public so that land's bisect can call it.
pub fn run_in_group(program: &Path, args: &[OsString], folder: &Path, limit: Duration) -> GroupRunEnd {
    decide_group_run_end(run_formatter_child(program, args, folder, limit))
}
