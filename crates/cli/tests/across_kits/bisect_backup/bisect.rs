//! CLI run_bisect/parse_args/report joined to land::bisect and real published Land history, Git probes and locks.
//! Probe results come from literal repository bytes or explicit process outcomes; no elapsed-time assertions.
use super::fixture::Harness;
use crate::{json::{check, Report}, support::{raw, script}};
use std::{fs, path::{Path, PathBuf}, process::Command};

fn byte_probe(s: &Harness) -> (PathBuf, PathBuf) {
    let program = s.f.root.join("byte-probe.sh"); let trace = s.f.root.join("probed-folders");
    script(&program, b"#!/bin/sh\nset -eu\nprintf '%s\\n' \"$PWD\" >>\"$1\"\nexec grep -qx base src/file\n");
    (program, trace)
}
fn run(s: &Harness, command: &mut Command, exit: i32, outcome: &str) -> Report {
    let before = s.observe(); let r = check(command.output().unwrap(), "bisect", exit, outcome);
    assert_eq!(s.observe(), before, "bisect must not change refs, op log, existing files/indexes or worktree registrations");
    s.assert_no_probes_left(); r
}
fn removed_trace(trace: &Path, probes: usize) {
    let text = fs::read_to_string(trace).unwrap(); let paths: Vec<_> = text.lines().collect();
    assert_eq!(paths.len(), probes);
    for path in &paths { assert!(!Path::new(path).exists(), "probe folder survived: {path}"); }
    let mut unique = paths.clone(); unique.sort(); unique.dedup(); assert_eq!(unique.len(), paths.len());
}

#[test]
fn fast_forward_and_merge_culprits_name_the_actual_landing_squash_and_zone_and_remove_every_probe() {
    for merge in [false, true] {
        let s = Harness::new(); let (squash, failed) = s.failing_landing(merge); let (program, trace) = byte_probe(&s);
        let mut c = s.bisect_command(&s.f.base, &failed, "60", &program, &[trace.to_str().unwrap()]);
        let r = run(&s, &mut c, 0, "culprit");
        assert_eq!(r.text("commit"), failed.as_str()); assert_eq!(r.text("change_id"), squash.change_id.as_str()); assert_eq!(r.text("zone"), "core");
        assert_eq!(r.strings("lines")[0], format!("the first failing landing is {failed}"));
        assert_eq!(r.strings("lines")[1], format!("its squash is change {} from zone core", squash.change_id));
        assert_eq!(r.text("next"), "fix the break in zone core through a request (mr folder request core <request> <path> --paths <path>... --actor <name>), then mr squash core --actor <name>");
        assert_eq!(r.strings("probes")[0], format!("{failed}: failed with exit 1"));
        assert_eq!(r.strings("probes")[1], format!("{}: passed", s.f.base));
        removed_trace(&trace, if merge { 3 } else { 2 });
        assert_eq!(r.strings("probes").len(), if merge { 3 } else { 2 });
    }
}

#[test]
fn a_real_commit_without_mortar_trailers_is_named_without_inventing_a_change_id_or_zone() {
    let s = Harness::new(); let failed = s.f.commit(&s.f.base, &[("src/file", b"broken\n")], "ordinary commit, no trailers\n");
    raw(&s.f.git, &["update-ref", "refs/heads/main", failed.as_str()]);
    let (program, trace) = byte_probe(&s);
    let r = run(&s, &mut s.bisect_command(&s.f.base, &failed, "60", &program, &[trace.to_str().unwrap()]), 0, "culprit");
    assert_eq!(r.text("commit"), failed.as_str()); assert!(r.is_null("change_id")); assert!(r.is_null("zone"));
    assert_eq!(r.strings("lines")[1], "it carries no squash trailers, so no change ID or zone is named");
    assert_eq!(r.text("next"), "fix the break in the zone that owns it through a request (mr folder request <zone> <request> <path> --paths <path>... --actor <name>), then mr squash <zone> --actor <name>");
    removed_trace(&trace, 2);
}

#[test]
fn both_not_found_reasons_preserve_history_and_remove_probe_worktrees() {
    for passes in [true, false] {
        let s = Harness::new(); let (_, failed) = s.failing_landing(false);
        let command = if passes { "exit 0" } else { "exit 1" };
        let r = run(&s, &mut s.bisect_command(&s.f.base, &failed, "60", Path::new("/bin/sh"), &["-c", command]), 3, "not-found");
        assert_eq!(r.text("last_pass"), s.f.base.as_str()); assert_eq!(r.text("failed"), failed.as_str());
        assert_eq!(r.text("owner"), "the test command");
        if passes {
            assert_eq!(r.strings("lines")[0], format!("no culprit: the failed commit {failed} passes the test command"));
            assert_eq!(r.text("next"), format!("check that the test command fails at {failed}, then repeat the same command"));
            assert_eq!(r.strings("probes"), &[format!("{failed}: passed")]);
        } else {
            assert_eq!(r.strings("lines")[0], format!("no culprit: the last passing commit {} fails the test command, so the break is at or before it", s.f.base));
            assert_eq!(r.text("next"), format!("repeat with an earlier passing commit: mr bisect <last pass> {failed} --timeout <seconds> --actor <name> -- <command>..."));
            assert_eq!(r.strings("probes"), &[format!("{failed}: failed with exit 1"), format!("{}: failed with exit 1", s.f.base)]);
        }
    }
}

#[test]
fn last_pass_off_the_first_parent_line_and_failed_off_origin_are_refused_before_running_any_command() {
    for last_off in [true, false] {
        let s = Harness::new(); let (_, failed) = s.failing_landing(true);
        let off = s.f.commit(&s.f.base, &[("src/off", b"off the line\n")], "side commit\n");
        let marker = s.f.root.join("must-not-run"); let program = s.f.root.join("marker.sh");
        script(&program, b"#!/bin/sh\nprintf 'ran\\n' >\"$1\"\nexit 0\n");
        let (last, end) = if last_off { (&off, &failed) } else { (&s.f.base, &off) };
        let r = run(&s, &mut s.bisect_command(last, end, "60", &program, &[marker.to_str().unwrap()]), 3, "refused");
        assert_eq!(r.text("owner"), "the commits named");
        let expected = if last_off { format!("the last passing commit {off} is not on {failed}'s first-parent line, so the range names no landings") }
        else { format!("the failed commit {off} is not on main's first-parent line, so it names no landing") };
        assert_eq!(r.strings("lines"), &[expected]); assert!(!marker.exists());
    }
}

#[test]
fn a_command_that_cannot_start_stops_at_the_failed_commit_and_leaves_no_probe_or_bisect_state() {
    let s = Harness::new(); let (_, failed) = s.failing_landing(false); let missing = s.f.root.join("no-such-program");
    assert!(!missing.exists());
    let r = run(&s, &mut s.bisect_command(&s.f.base, &failed, "60", &missing, &[]), 3, "refused");
    assert_eq!(r.strings("lines"), &[format!("the test command could not be run at {failed}, so its verdict there is unknown and the search stopped")]);
    assert_eq!(r.text("owner"), "the test command");
    assert_eq!(r.text("next"), "check that the test command's program exists and can be run, then repeat the same command");
}

#[test]
fn actor_after_double_dash_is_passed_to_each_probe_not_consumed_as_a_cli_global() {
    let s = Harness::new(); let (_, failed) = s.failing_landing(false);
    let program = s.f.root.join("args-probe.sh"); let trace = s.f.root.join("probe-args");
    script(&program, b"#!/bin/sh\nset -eu\ntrace=$1; shift\nprintf '%s\\n' \"$@\" >>\"$trace\"\n[ \"$#\" -eq 2 ] && [ \"$1\" = --actor ] && [ \"$2\" = probe-owner ] || exit 17\nexec grep -qx base src/file\n");
    let r = run(&s, &mut s.bisect_command(&s.f.base, &failed, "60", &program, &[trace.to_str().unwrap(), "--actor", "probe-owner"]), 0, "culprit");
    assert_eq!(r.text("commit"), failed.as_str()); assert_eq!(fs::read(&trace).unwrap(), b"--actor\nprobe-owner\n--actor\nprobe-owner\n");
}

#[test]
fn a_fifo_blocked_probe_is_reported_timed_out_and_counted_as_a_failure_without_a_speed_oracle() {
    let s = Harness::new(); let (_, failed) = s.failing_landing(false);
    let fifo = s.f.root.join("blocked-probe"); let pid_file = s.f.root.join("blocked-probe.pid"); let program = s.f.root.join("timeout-probe.sh");
    let made = Command::new("mkfifo").arg(&fifo).output().unwrap(); assert!(made.status.success(), "{made:?}");
    script(&program, b"#!/bin/sh\nset -eu\nprintf '%s\\n' \"$$\" >>\"$2\"\nif grep -qx base src/file; then exit 0; fi\nexec /bin/cat \"$1\"\n");
    let r = run(&s, &mut s.bisect_command(&s.f.base, &failed, "1", &program, &[fifo.to_str().unwrap(), pid_file.to_str().unwrap()]), 0, "culprit");
    assert_eq!(r.text("commit"), failed.as_str());
    assert_eq!(r.strings("probes"), &[format!("{failed}: timed out after 1 s, counted as a fail"), format!("{}: passed", s.f.base)]);
    assert!(r.strings("lines").contains(&format!("probe {failed}: timed out after 1 s, counted as a fail")));
    let pids: Vec<u32> = fs::read_to_string(pid_file).unwrap().lines().map(|l| l.parse().unwrap()).collect();
    assert!(matches!(pids.len(), 1 | 2), "one pid per probe that started, failed probe first: {pids:?}");
    // One line: the timed-out probe was stopped before its first command, so it never reached its blocking read.
    if let [pid, _] = pids[..] { assert!(!oplog::PidProbe::pid_alive(&oplog::KillPidProbe, pid), "timed-out probe must be stopped and reaped"); }
}
