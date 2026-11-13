//! A real-git crash boundary, paired with run_with_kill_gate. Covered by tests/kill_gate.rs (S-5).

use crate::write_git_shim;
use std::io;
use std::path::Path;

/// Writes a git wrapper that sends SIGKILL before/after update-ref, then blocks until its
/// outer parent has reaped the worker. Signal submission alone is not a crash boundary.
/// The worker supplies MORTAR_TEST_WORKER_PID and MORTAR_TEST_CRASH (before/after).
pub fn write_git_kill_shim(path: &Path) -> io::Result<()> {
    write_git_shim(path, br#"#!/bin/sh
set -eu
if [ "${3-}" != update-ref ]; then exec git "$@"; fi
[ "$PPID" = "$MORTAR_TEST_WORKER_PID" ] || exit 98
case "$MORTAR_TEST_CRASH" in
  before) ;;
  after) git "$@" ;;
  *) exit 99 ;;
esac
printf 'armed\n' > "$TESTKIT_GATE_ARMED"
kill -KILL "$MORTAR_TEST_WORKER_PID"
# Keep our output pipes open until the outer test has observed and reaped worker death.
IFS= read -r release < "$TESTKIT_GATE_RELEASE"
[ "$release" = release ] || exit 97
# A live worker or zombie still has a pid; only parent reaping removes it.
if kill -0 "$MORTAR_TEST_WORKER_PID" 2>/dev/null; then
  printf 'not-reaped\n' > "$TESTKIT_GATE_ACK"
else
  printf 'reaped\n' > "$TESTKIT_GATE_ACK"
fi
"#)
}
