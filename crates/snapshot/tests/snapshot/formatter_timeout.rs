//! Deadline outcomes, real child death/reaping and capped stderr.
//! No elapsed-time assertions; no background children, held-pipe case or process-group behavior.
use crate::support::{assert_snapshot_written, blob, fifo_hang_guard, formatter_script, write, Fixture};
use oplog::{KillPidProbe, PidProbe};
use snapshot::{snapshot_formatted, Formatter, FormatterResult, DEFAULT_SIZE_CAP};
use std::{fs, path::PathBuf, process::Command, time::Duration};
use testkit::write_git_shim;

#[test]
fn fifo_blocked_formatter_is_timed_out_with_limit_and_stderr_killed_reaped_and_snapshot_logged() {
    // Joins snapshot_formatted -> actual blocked child -> timeout kill/wait -> PID probe -> real gitio/oplog save.
    let f = Fixture::new(&[]);
    write(&f.repo.path, "root.txt", b"save after timeout\n");
    let fifo = f.common.join("never-released-fifo");
    let made = Command::new("mkfifo").arg(&fifo).output().unwrap();
    assert!(made.status.success(), "{}", String::from_utf8_lossy(&made.stderr));
    let guard_fired = f.common.join("fifo-hang-guard-fired");
    let _hang_guard = fifo_hang_guard(vec![fifo.clone()], guard_fired.clone(), Duration::from_secs(30));
    let pid_file = f.common.join("timeout-formatter.pid");
    let _cleanup = StopFormatterOnDrop { pid_file: pid_file.clone() };
    let program = f.common.join("block-on-fifo.sh");
    // exec retains the recorded PID. No child/grandchild is left holding stderr: cat itself blocks
    // opening a FIFO with no writer until the product kills it, rather than a fixture sleeping on a clock.
    write_git_shim(&program, b"#!/bin/sh\nset -eu\nprintf '%s\\n' \"$$\" >\"$1\"\nprintf 'formatter stalled on FIFO\\n' >&2\nexec /bin/cat \"$2\"\n").unwrap();
    let limit = Duration::from_secs(2);
    let formatter = Formatter { program, args: vec![pid_file.clone().into_os_string(), fifo.into_os_string(), "--".into()], timeout: limit };
    let index_before = fs::read(f.common.join("index")).unwrap();
    let saved = snapshot_formatted(&f.git, &f.repo.path, &f.target, &[], DEFAULT_SIZE_CAP, Some(&formatter)).unwrap();
    assert_snapshot_written(&f, &saved, &f.head);
    let run = saved.formatted.as_ref().unwrap();
    assert_eq!(run.program, formatter.program);
    assert_eq!(run.args, formatter.args);
    assert_eq!(run.paths, vec![PathBuf::from("root.txt")]);
    assert!(!guard_fired.exists(), "hang guard had to unblock the FIFO: kill/reap regressed");
    assert_eq!(run.result, FormatterResult::TimedOut { limit, stderr: "formatter stalled on FIFO\n".into() });
    let pid = formatter_pid(&pid_file);
    assert_ne!(pid, std::process::id());
    // kill(pid,0) sees zombies as alive, so absence proves both termination and reaping, not just signal delivery.
    assert!(!KillPidProbe.pid_alive(pid), "formatter {pid} is still running or unreaped");
    assert_eq!(blob(&f.git, &saved.tree, "root.txt"), b"save after timeout\n");
    assert_eq!(fs::read(f.repo.path.join("root.txt")).unwrap(), b"save after timeout\n");
    assert_eq!(fs::read(f.common.join("index")).unwrap(), index_before);
}

#[test]
fn normal_formatter_exits_within_its_limit_and_is_reaped_without_timeout_result() {
    // Joins migrated required timeout -> real successful formatter -> normal wait/reap -> formatted snapshot/log.
    let f = Fixture::new(&[]);
    write(&f.repo.path, "root.txt", b"normal formatter input\n");
    let (formatter, trace) = formatter_script(&f, b"printf '%s\\n' \"$$\" >\"$trace.pid\"\nfor path in \"$@\"; do printf 'normal formatted output\\n' >\"$path\"; done\n");
    let _cleanup = StopFormatterOnDrop { pid_file: trace.with_extension("pid") };
    let saved = snapshot_formatted(&f.git, &f.repo.path, &f.target, &[], DEFAULT_SIZE_CAP, Some(&formatter)).unwrap();
    assert_snapshot_written(&f, &saved, &f.head);
    assert_eq!(saved.formatted.as_ref().unwrap().result, FormatterResult::Formatted);
    assert_eq!(saved.formatted.as_ref().unwrap().paths, vec![PathBuf::from("root.txt")]);
    let pid = formatter_pid(&trace.with_extension("pid"));
    assert_ne!(pid, std::process::id());
    assert!(!KillPidProbe.pid_alive(pid), "normal formatter {pid} was not reaped");
    assert_eq!(blob(&f.git, &saved.tree, "root.txt"), b"normal formatted output\n");
    assert_eq!(fs::read(trace.with_extension("count")).unwrap(), b"run\n");
}

#[test]
fn stderr_larger_than_pipe_and_cap_is_drained_but_only_first_4096_bytes_are_reported() {
    // Joins real pipe backpressure -> capped concurrent drain -> Failed exit/stderr -> snapshot still written/logged.
    let f = Fixture::new(&[]);
    write(&f.repo.path, "root.txt", b"save after noisy failure\n");
    let (formatter, trace) = formatter_script(&f, b"cat \"$trace.payload\" >&2\nexit 1\n");
    let mut payload = vec![b'A'; 4096];
    payload.extend(vec![b'B'; 256 * 1024]);
    fs::write(trace.with_extension("payload"), payload).unwrap();
    let saved = snapshot_formatted(&f.git, &f.repo.path, &f.target, &[], DEFAULT_SIZE_CAP, Some(&formatter)).unwrap();
    assert_snapshot_written(&f, &saved, &f.head);
    assert_eq!(saved.formatted.as_ref().unwrap().result, FormatterResult::Failed { code: Some(1), stderr: "A".repeat(4096) });
    assert_eq!(blob(&f.git, &saved.tree, "root.txt"), b"save after noisy failure\n");
    assert_eq!(fs::read(trace.with_extension("count")).unwrap(), b"run\n");
}

fn formatter_pid(path: &std::path::Path) -> u32 {
    fs::read_to_string(path).unwrap().trim().parse().unwrap()
}

/// Best-effort fixture cleanup if incorrect code returns without killing its still-live formatter.
struct StopFormatterOnDrop { pid_file: PathBuf }
impl Drop for StopFormatterOnDrop {
    fn drop(&mut self) {
        let Some(pid) = fs::read_to_string(&self.pid_file).ok().and_then(|s| s.trim().parse::<u32>().ok()) else { return; };
        if pid != std::process::id() && KillPidProbe.pid_alive(pid) {
            let _ = Command::new("kill").args(["-KILL", &pid.to_string()]).status();
        }
    }
}
