//! Real shell descendants, held stderr and post-exit write fencing.
//! FIFO control/EOF and kernel process state prove outcomes; no elapsed-time assertion, sleep or polling.
use crate::support::{assert_snapshot_written, blob, fifo_hang_guard, write, Fixture};
use oplog::{KillPidProbe, PidProbe};
use snapshot::{snapshot_formatted, Formatter, FormatterResult, DEFAULT_SIZE_CAP};
use std::{fs::{self, File, OpenOptions}, io::{BufRead, BufReader, Read, Write}, path::{Path, PathBuf}, process::Command, thread, time::Duration};
use testkit::write_git_shim;

#[test]
fn wrapper_and_its_fifo_blocked_child_are_terminated_on_timeout_and_snapshot_is_written() {
    // Joins snapshot_formatted -> sh wrapper/real background sh -> group kill -> EOF/process state -> gitio/oplog.
    exercise_group("timeout", Duration::from_secs(2));
}

#[test]
fn normally_exited_leader_kills_child_holding_stderr_without_external_release_or_timeout_result() {
    // Joins WNOWAIT normal-exit handling -> group kill closes inherited stderr -> Formatted and durable snapshot.
    // A failure-only watchdog releases the child with RELEASED data; accepting that path cannot pass this test.
    exercise_group("normal", Duration::from_secs(60));
}

#[test]
fn background_write_released_after_leader_exit_cannot_enter_snapshot_tree() {
    // Joins normal-exit group cleanup -> post-formatter real read-tree gate -> attempted late write -> saved tree/log.
    let f = Fixture::new(&[]);
    write(&f.repo.path, "root.txt", b"safe before background writer\n");
    let group = group_fixture(&f, "write", Duration::from_secs(60));
    let _guard = fifo_hang_guard(group.fifos(), group.fired.clone(), Duration::from_secs(30));
    let shim = f.common.join("post-formatter-tree.sh");
    let once = f.common.join("post-formatter-tree-once");
    write_git_shim(&shim, b"#!/bin/sh\nset -eu\nif [ \"$3\" = read-tree ] && [ -f \"$MORTAR_GROUP_EXITING\" ] && mkdir \"$MORTAR_GROUP_ONCE\" 2>/dev/null; then\n  exec 5<>\"$MORTAR_GROUP_RESUME\"\n  printf 'TREE\\n' >\"$MORTAR_GROUP_BOUNDARY\"\n  IFS= read -r resume <&5\n  exec 5>&-\nfi\nexec git \"$@\"\n").unwrap();
    let mut git = f.git.clone();
    git.program = shim;
    git.env.extend([
        ("MORTAR_GROUP_EXITING".into(), group.exiting.clone().into_os_string()),
        ("MORTAR_GROUP_ONCE".into(), once.clone().into_os_string()),
        ("MORTAR_GROUP_BOUNDARY".into(), group.boundary.clone().into_os_string()),
        ("MORTAR_GROUP_RESUME".into(), group.resume.clone().into_os_string()),
    ]);
    let index_before = fs::read(f.common.join("index")).unwrap();
    let (saved, pid, pgid, tail, boundary) = thread::scope(|scope| {
        let job = scope.spawn(|| snapshot_formatted(&git, &f.repo.path, &f.target, &[], DEFAULT_SIZE_CAP, Some(&group.formatter)));
        let (mut channel, pid, pgid) = child_channel(&group.death);
        let boundary = fs::read_to_string(&group.boundary).unwrap();
        // The first pre-formatter tree was already built. This gate is before the subsequent read-tree,
        // after the formatter leader exited. A surviving child receives GO and writes before git may continue.
        release(&group.gate, b"GO\n");
        let mut tail = String::new();
        channel.read_to_string(&mut tail).unwrap();
        release(&group.resume, b"CONTINUE\n");
        (job.join().unwrap().unwrap(), pid, pgid, tail, boundary)
    });
    assert_eq!(boundary, "TREE\n");
    assert!(once.exists());
    assert!(!group.fired.exists(), "watchdog had to release a stalled fixture");
    assert_eq!(tail, "", "surviving background child ran after the leader exited");
    assert_group_terminated(&group, pid, pgid);
    assert_eq!(saved.formatted.as_ref().unwrap().result, FormatterResult::Formatted);
    assert_snapshot_written(&f, &saved, &f.head);
    assert_eq!(blob(&f.git, &saved.tree, "root.txt"), b"safe before background writer\n");
    assert_eq!(fs::read(f.repo.path.join("root.txt")).unwrap(), b"safe before background writer\n");
    assert_eq!(fs::read(f.common.join("index")).unwrap(), index_before);
}

#[test]
fn hang_guard_marks_its_intervention_and_unblocks_a_fifo_without_a_formatter_kill() {
    // Joins the failure-only guard -> real FIFO wakeup -> observable marker, so the guard is not an untested escape.
    let f = Fixture::new(&[]);
    let fifo = f.common.join("guard-self-check");
    make_fifos(std::slice::from_ref(&fifo));
    // Hold a reader before the immediate guard fires, so its message remains observable regardless of scheduling.
    let reader = OpenOptions::new().read(true).write(true).open(&fifo).unwrap();
    let fired = f.common.join("guard-self-check-fired");
    let guard = fifo_hang_guard(vec![fifo], fired.clone(), Duration::ZERO);
    let mut line = String::new();
    BufReader::new(reader).read_line(&mut line).unwrap();
    assert_eq!(line, "WATCHDOG\n");
    assert_eq!(fs::read(fired).unwrap(), b"hang guard released fixture\n");
    drop(guard);
}

fn exercise_group(mode: &str, limit: Duration) {
    let f = Fixture::new(&[]);
    write(&f.repo.path, "root.txt", b"save after group cleanup\n");
    let group = group_fixture(&f, mode, limit);
    let _guard = fifo_hang_guard(group.fifos(), group.fired.clone(), Duration::from_secs(30));
    let index_before = fs::read(f.common.join("index")).unwrap();
    let (saved, pid, pgid, tail) = thread::scope(|scope| {
        let job = scope.spawn(|| snapshot_formatted(&f.git, &f.repo.path, &f.target, &[], DEFAULT_SIZE_CAP, Some(&group.formatter)));
        let (mut channel, pid, pgid) = child_channel(&group.death);
        let mut tail = String::new();
        channel.read_to_string(&mut tail).unwrap();
        (job.join().unwrap().unwrap(), pid, pgid, tail)
    });
    assert!(!group.fired.exists(), "watchdog had to release a surviving child");
    assert_eq!(tail, "", "child exited only after external release, rather than group cleanup");
    assert_group_terminated(&group, pid, pgid);
    let expected = if mode == "timeout" {
        FormatterResult::TimedOut { limit, stderr: "wrapper child ready\n".into() }
    } else { FormatterResult::Formatted };
    assert_eq!(saved.formatted.as_ref().unwrap().result, expected);
    assert_snapshot_written(&f, &saved, &f.head);
    assert_eq!(blob(&f.git, &saved.tree, "root.txt"), b"save after group cleanup\n");
    assert_eq!(fs::read(f.repo.path.join("root.txt")).unwrap(), b"save after group cleanup\n");
    assert_eq!(fs::read(f.common.join("index")).unwrap(), index_before);
}

/// Direct run_in_group caller, sharing the proven real FIFO/group fixture.
pub(super) fn exercise_public_runner(mode: &str, limit: Duration) {
    let f = Fixture::new(&[]);
    let group = group_fixture(&f, mode, limit);
    let _guard = fifo_hang_guard(group.fifos(), group.fired.clone(), Duration::from_secs(30));
    let (answer, pid, pgid, tail) = thread::scope(|scope| {
        let job = scope.spawn(|| snapshot::run_in_group(&group.formatter.program, &group.formatter.args, &f.repo.path, limit));
        let (mut channel, pid, pgid) = child_channel(&group.death);
        let mut tail = String::new();
        channel.read_to_string(&mut tail).unwrap();
        (job.join().unwrap(), pid, pgid, tail)
    });
    assert!(!group.fired.exists(), "failure-only guard released a surviving child");
    assert_eq!(tail, "", "child survived group cleanup until an external release");
    assert_group_terminated(&group, pid, pgid);
    let expected = if mode == "timeout" {
        snapshot::GroupRunEnd::TimedOut { limit, stderr: "wrapper child ready\n".into() }
    } else {
        snapshot::GroupRunEnd::Exited { code: Some(0), stderr: "wrapper child ready\n".into() }
    };
    assert_eq!(answer, expected, "direct runner keeps raw successful stderr, unlike formatter mapping");
}

struct Group {
    formatter: Formatter, death: PathBuf, ready: PathBuf, gate: PathBuf,
    boundary: PathBuf, resume: PathBuf, exiting: PathBuf, leader: PathBuf, fired: PathBuf,
}
impl Group {
    fn fifos(&self) -> Vec<PathBuf> {
        vec![self.gate.clone(), self.ready.clone(), self.death.clone(), self.boundary.clone(), self.resume.clone()]
    }
}

fn group_fixture(f: &Fixture, mode: &str, limit: Duration) -> Group {
    let path = |name: &str| f.common.join(format!("group-{name}"));
    let group = Group {
        formatter: Formatter { program: path("wrapper.sh"), args: vec![], timeout: limit },
        death: path("death"), ready: path("ready"), gate: path("gate"), boundary: path("boundary"),
        resume: path("resume"), exiting: path("exiting"), leader: path("leader.pid"), fired: path("watchdog-fired"),
    };
    make_fifos(&group.fifos());
    // Only the background child owns fd3. It closes that descriptor only on exit: EOF without RELEASED/WROTE
    // proves it died without receiving permission to finish. The write-mode child closes stderr, so the
    // post-exit write case cannot pass merely because a held stderr pipe delayed the snapshot.
    write_git_shim(&group.formatter.program, br#"#!/bin/sh
set -eu
mode=$1; death=$2; ready=$3; gate=$4; exiting=$5; leader=$6; target=$7
printf '%s\n' "$$" >"$leader"
sh -c '
  set -eu
  mode=$1; ready=$2; gate=$3; target=$4
  if [ "$mode" = write ]; then exec 2>/dev/null; fi
  exec 4<>"$gate"
  pgid=$(ps -p "$$" -o pgid=)
  printf "%s %s\n" "$$" "$pgid" >&3
  printf "READY\n" >"$ready"
  IFS= read -r signal <&4
  if [ "$mode" = write ]; then
    printf "background mutation\n" >"$target"
    printf "WROTE\n" >&3
  else
    printf "RELEASED\n" >&3
  fi
' sh "$mode" "$ready" "$gate" "$target" 3>"$death" &
child=$!
IFS= read -r message <"$ready"
printf 'wrapper child ready\n' >&2
printf 'EXITING\n' >"$exiting"
if [ "$mode" = timeout ]; then wait "$child"; else exit 0; fi
"#).unwrap();
    let mut group = group;
    group.formatter.args = vec![mode.into(), group.death.clone().into_os_string(), group.ready.clone().into_os_string(),
        group.gate.clone().into_os_string(), group.exiting.clone().into_os_string(), group.leader.clone().into_os_string(),
        f.repo.path.join("root.txt").into_os_string(), "--".into()];
    group
}

fn child_channel(path: &Path) -> (BufReader<File>, u32, u32) {
    let mut channel = BufReader::new(File::open(path).unwrap());
    let mut line = String::new();
    channel.read_line(&mut line).unwrap();
    let fields: Vec<u32> = line.split_whitespace().map(|s| s.parse().unwrap()).collect();
    assert_eq!(fields.len(), 2, "child did not report PID and PGID: {line}");
    (channel, fields[0], fields[1])
}

fn assert_group_terminated(group: &Group, child: u32, pgid: u32) {
    let leader: u32 = fs::read_to_string(&group.leader).unwrap().trim().parse().unwrap();
    assert_ne!(leader, std::process::id());
    assert_ne!(child, leader);
    assert_eq!(pgid, leader, "formatter must lead a new group inherited by its real child");
    assert!(!KillPidProbe.pid_alive(leader), "leader {leader} is still running or unreaped");
    // The child's only write end of the death FIFO is its fd3, which the script never closes, so the EOF each
    // caller reads before calling this helper means the child has ended. With each caller's empty tail, it ended
    // without writing RELEASED or WROTE, that is, without the release.
}

fn make_fifos(paths: &[PathBuf]) {
    let output = Command::new("mkfifo").args(paths).output().unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
}

fn release(path: &Path, bytes: &[u8]) {
    let mut file = OpenOptions::new().read(true).write(true).open(path).unwrap();
    file.write_all(bytes).unwrap();
}
