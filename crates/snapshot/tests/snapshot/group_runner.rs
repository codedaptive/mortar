//! Public run_in_group -> real spawn, literal argv/cwd, raw stderr and kernel reap/group outcomes.
use crate::support::Fixture;
use oplog::{KillPidProbe, PidProbe};
use snapshot::{run_in_group, GroupRunEnd};
use std::{ffi::OsString, fs, path::Path, time::Duration};
use testkit::write_git_shim;

#[test]
fn public_runner_preserves_literal_args_cwd_and_successful_stderr_and_reaps_child() {
    let f = Fixture::new(&[]);
    let program = f.common.join("public-runner.sh");
    let trace = f.common.join("public-runner.args");
    let pid = f.common.join("public-runner.pid");
    write_git_shim(&program, b"#!/bin/sh\nset -eu\ntrace=$1; pid=$2; shift 2\nprintf '%s' \"$$\" >\"$pid\"\nprintf '%s\\0' \"$@\" >\"$trace\"\nprintf 'cwd marker\\n' > runner-cwd\nif IFS= read -r line; then exit 91; fi\nprintf 'stdout is discarded\\n'\nprintf 'successful warning\\377' >&2\n").unwrap();
    let args = vec![trace.clone().into_os_string(), pid.clone().into_os_string(), "space value".into(),
        "$(touch MUST_NOT_EXIST)".into(), "*".into(), "line\nbreak".into(), "--".into()];
    let before_refs = f.git.list_refs("").unwrap();
    let before_log = fs::read(oplog::oplog_path(&f.common)).ok();
    let answer = run_in_group(&program, &args, &f.repo.path, Duration::from_secs(60));
    assert_eq!(answer, GroupRunEnd::Exited { code: Some(0), stderr: "successful warning\u{fffd}".into() });
    assert_eq!(fs::read(trace).unwrap(), b"space value\0$(touch MUST_NOT_EXIST)\0*\0line\nbreak\0--\0");
    assert_eq!(fs::read(f.repo.path.join("runner-cwd")).unwrap(), b"cwd marker\n");
    assert!(!f.repo.path.join("MUST_NOT_EXIST").exists());
    assert_reaped(&pid);
    assert_eq!(f.git.list_refs("").unwrap(), before_refs);
    assert_eq!(fs::read(oplog::oplog_path(&f.common)).ok(), before_log);
}

#[test]
fn public_runner_drains_pipe_and_caps_raw_stderr_without_changing_the_exit_code() {
    let f = Fixture::new(&[]);
    let program = f.common.join("public-noisy.sh");
    let payload = f.common.join("public-noisy.payload");
    let pid = f.common.join("public-noisy.pid");
    let mut bytes = vec![b'A'; 4096]; bytes.extend(vec![b'B'; 256 * 1024]);
    fs::write(&payload, bytes).unwrap();
    write_git_shim(&program, b"#!/bin/sh\nprintf '%s' \"$$\" >\"$2\"\ncat \"$1\" >&2\nexit 23\n").unwrap();
    assert_eq!(run_in_group(&program, &[payload.into_os_string(), pid.clone().into_os_string()], &f.repo.path, Duration::from_secs(60)),
        GroupRunEnd::Exited { code: Some(23), stderr: "A".repeat(4096) });
    assert_reaped(&pid);
}

#[test]
fn public_runner_reports_signal_exit_and_missing_program_or_cwd_as_distinct_outcomes() {
    let f = Fixture::new(&[]);
    let program = f.common.join("public-signalled.sh");
    let pid = f.common.join("public-signalled.pid");
    write_git_shim(&program, b"#!/bin/sh\nprintf '%s' \"$$\" >\"$1\"\nprintf 'before signal\\n' >&2\nkill -TERM \"$$\"\n").unwrap();
    assert_eq!(run_in_group(&program, &[pid.clone().into_os_string()], &f.repo.path, Duration::from_secs(60)),
        GroupRunEnd::Exited { code: None, stderr: "before signal\n".into() });
    assert_reaped(&pid);
    for (program, folder) in [(f.common.join("no-such-program"), f.repo.path.clone()), ("/bin/sh".into(), f.repo.path.join("missing-cwd"))] {
        let GroupRunEnd::SpawnFailed { error } = run_in_group(&program, &Vec::<OsString>::new(), &folder, Duration::from_secs(60)) else { panic!("expected spawn failure") };
        assert!(error.contains("No such file or directory"), "{error}");
    }
}

#[test]
fn public_runner_timeout_kills_and_reaps_the_leader_and_its_fifo_blocked_child() {
    // Allow startup under load to reach the hang; the FIFO, not the clock, holds it.
    super::formatter_group::exercise_public_runner("timeout", Duration::from_secs(20));
}

#[test]
fn public_runner_normal_exit_kills_child_holding_stderr_and_keeps_the_leaders_warning() {
    super::formatter_group::exercise_public_runner("normal", Duration::from_secs(60));
}

fn assert_reaped(path: &Path) {
    let pid = fs::read_to_string(path).unwrap().parse().unwrap();
    assert_ne!(pid, std::process::id());
    assert!(!KillPidProbe.pid_alive(pid), "process is running or unreaped");
}
