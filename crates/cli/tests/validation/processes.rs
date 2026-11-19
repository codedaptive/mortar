use crate::support::*;
use crate::support::FormatterConfig;
use oplog::{KillPidProbe, PidProbe};
use std::{fs::{self, File, OpenOptions}, io::{BufRead, BufReader, Write}, os::unix::process::ExitStatusExt,
    path::{Path, PathBuf}, process::{Child, Command}, sync::mpsc, thread, time::Duration};

#[test]
fn validation_configured_formatter_runs_once_before_save_or_join_and_its_bytes_enter_the_saved_commit() {
    for verb in ["save", "join"] {
        let f = Fixture::new(); let program = f.common.join("formatter.sh"); let trace = f.common.join("formatter.trace");
        script(&program, b"#!/bin/sh\nset -eu\ntrace=$1; shift\ntest \"$1\" = --; shift\nprintf 'run\\n' >>\"$trace\"\npwd >\"$trace.cwd\"\nprintf '%s\\0' \"$@\" >\"$trace.args\"\nfor path in \"$@\"; do printf 'formatted child output\\n' >\"$path\"; done\n");
        let mut config = f.init_args.clone(); config.formatter = Some(FormatterConfig { command: vec![program.to_str().unwrap().into(), trace.to_str().unwrap().into(), "--".into()], timeout_seconds: 60 }); f.configure(&config);
        f.write("src/file", b"unformatted input\n");
        let args = if verb == "save" { vec!["--json", "save", "Save formatted work"] } else { vec!["--json", "join"] };
        f.report(&args, verb, 0, if verb == "save" { "saved" } else { "joined" });
        assert_eq!(f.blob(&f.head(), "src/file"), b"formatted child output\n");
        assert_eq!(fs::read(f.request.join("src/file")).unwrap(), b"formatted child output\n");
        assert_eq!(fs::read(&trace).unwrap(), b"run\n");
        assert_eq!(fs::read(trace.with_extension("trace.args")).unwrap(), b"src/file\0");
        assert_eq!(fs::read_to_string(trace.with_extension("trace.cwd")).unwrap().trim_end(), f.request.to_str().unwrap());
        assert!(!f.lock_path().exists());
    }
}

#[test]
fn validation_fifo_formatter_timeout_is_reported_and_save_still_commits_work_then_reaps_formatter() {
    let f = Fixture::new(); let fifo = f.common.join("formatter-blocked"); make_fifos(std::slice::from_ref(&fifo));
    let fired = f.common.join("timeout-guard-fired"); let _guard = FifoGuard::new(vec![fifo.clone()], fired.clone());
    let pid_file = f.common.join("formatter.pid"); let _cleanup = StopPid(pid_file.clone()); let program = f.common.join("timeout.sh");
    script(&program, b"#!/bin/sh\nset -eu\nprintf '%s' \"$$\" >\"$1\"\nprintf 'blocked formatter\\n' >&2\nexec /bin/cat \"$2\"\n");
    let mut config = f.init_args.clone(); config.formatter = Some(FormatterConfig {
        command: vec![program.to_str().unwrap().into(), pid_file.to_str().unwrap().into(), fifo.to_str().unwrap().into()], timeout_seconds: 2 }); f.configure(&config);
    f.write("src/file", b"work saved after timeout\n");
    let report = f.report(&["--json", "save", "Save despite formatter timeout"], "save", 0, "saved");
    assert!(report.lines().contains("timeout of 2 s and was stopped")); assert!(!fired.exists(), "guard released a stuck process instead of product timeout");
    let pid = fs::read_to_string(pid_file).unwrap().parse().unwrap();
    assert!(!KillPidProbe.pid_alive(pid), "formatter must be dead AND reaped");
    assert_eq!(f.blob(&f.head(), "src/file"), b"work saved after timeout\n"); assert!(!f.lock_path().exists());
}

#[test]
fn validation_killed_cli_after_actual_rebase_transaction_recovers_then_moves_folder_before_save_once() {
    let f = Fixture::new(); let old = f.save_topic(); let zone = f.advance_zone(false);
    let dir = f.common.join("git-shim"); fs::create_dir(&dir).unwrap();
    let event = f.common.join("rebase-event"); let release = f.common.join("rebase-release"); make_fifos(&[event.clone(), release.clone()]);
    let fired = f.common.join("rebase-guard-fired"); let _guard = FifoGuard::new(vec![event.clone(), release.clone()], fired.clone());
    let once = f.common.join("rebase-gated-once");
    script(&dir.join("git"), br#"#!/bin/sh
set -eu
if [ "${3-}" = update-ref ] && [ "${4-}" = --stdin ] &&
   grep -q '"kind":"join-rebase"' "$MORTAR_VALIDATION_LOG" && mkdir "$MORTAR_VALIDATION_ONCE" 2>/dev/null; then
  "$MORTAR_VALIDATION_REAL_GIT" "$@"
  printf 'READY\n' >"$MORTAR_VALIDATION_EVENT"
  IFS= read -r signal <"$MORTAR_VALIDATION_RELEASE"
  [ "$signal" = release ] || exit 97
  exit 0
fi
exec "$MORTAR_VALIDATION_REAL_GIT" "$@"
"#);
    let mut command = f.command(&f.request, &["--json", "join"]);
    command.env("PATH", shim_path(&dir)).env("MORTAR_VALIDATION_REAL_GIT", actual_git())
        .env("MORTAR_VALIDATION_LOG", oplog::oplog_path(&f.common)).env("MORTAR_VALIDATION_ONCE", &once)
        .env("MORTAR_VALIDATION_EVENT", &event).env("MORTAR_VALIDATION_RELEASE", &release);
    let mut child = Process(command.spawn().unwrap());
    let mut line = String::new(); BufReader::new(File::open(&event).unwrap()).read_line(&mut line).unwrap();
    assert_eq!(line, "READY\n"); assert!(!fired.exists());
    let pid = child.0.id(); let holder = oplog::decode_lock_record(&fs::read_to_string(f.lock_path()).unwrap()).unwrap(); assert_eq!(holder.pid, pid);
    let new = f.head(); assert_ne!(new, old); assert_eq!(f.git.read_ref(ZONE).unwrap(), Some(zone.clone()));
    assert_eq!(f.git.commit_info(&new).unwrap().parents, vec![zone]);
    assert_eq!(f.blob(&new, "src/file"), b"topic work\n"); assert_eq!(f.blob(&new, "src/upstream"), b"upstream work\n");
    assert!(!f.request.join("src/upstream").exists(), "the app has not moved the folder yet");
    child.0.kill().unwrap(); assert_eq!(child.0.wait().unwrap().signal(), Some(9));
    assert!(!KillPidProbe.pid_alive(pid), "kernel wait/reap BEFORE releasing the git shim");
    release_fifo(&release, b"release\n");
    let records = f.records();
    let intent = records.iter().find(|r| r.kind == "join-rebase" && r.phase == oplog::OpPhase::Intent).unwrap();
    assert_eq!(oplog::open_intent(&f.common, &intent.op_id).unwrap(), Some(intent.moves.clone()));
    let report = f.report(&["--json", "join"], "join", 0, "joined");
    assert_eq!(report.text("finished_move_from"), old.as_str()); assert_eq!(report.text("finished_move_to"), new.as_str());
    assert!(!report.flag("saved_written"), "folder catch-up must happen BEFORE saving, not save the old folder as new work");
    assert_eq!(f.head(), new); assert_eq!(fs::read(f.request.join("src/upstream")).unwrap(), b"upstream work\n");
    assert_eq!(f.records().iter().filter(|r| r.kind == "join-rebase" && r.phase == oplog::OpPhase::Done).count(), 1);
    assert_eq!(oplog::open_intent(&f.common, &intent.op_id).unwrap(), None); assert!(!f.lock_path().exists()); assert!(!fired.exists());
    let before = f.observe(); f.report(&["--json", "join"], "join", 0, "joined"); assert_eq!(f.observe(), before);
}

fn make_fifos(paths: &[PathBuf]) {
    let result = Command::new("mkfifo").args(paths).output().unwrap(); assert!(result.status.success(), "{result:?}");
}
fn release_fifo(path: &Path, bytes: &[u8]) {
    if let Ok(mut fifo) = OpenOptions::new().read(true).write(true).open(path) { let _ = fifo.write_all(bytes); }
}
struct FifoGuard { cancel: Option<mpsc::Sender<()>>, thread: Option<thread::JoinHandle<()>>, fifos: Vec<PathBuf> }
impl FifoGuard {
    fn new(fifos: Vec<PathBuf>, marker: PathBuf) -> Self {
        let (send, recv) = mpsc::channel(); let paths = fifos.clone();
        let job = thread::spawn(move || { if recv.recv_timeout(Duration::from_secs(30)).is_err() {
            fs::write(marker, b"failure guard released FIFO\n").unwrap(); for path in paths { release_fifo(&path, b"WATCHDOG\n"); }
        } });
        Self { cancel: Some(send), thread: Some(job), fifos }
    }
}
impl Drop for FifoGuard { fn drop(&mut self) {
    for path in &self.fifos { release_fifo(path, b"STOP\n"); }
    if let Some(send) = self.cancel.take() { let _ = send.send(()); }
    if let Some(job) = self.thread.take() { let _ = job.join(); }
} }
struct Process(Child);
impl Drop for Process { fn drop(&mut self) { if self.0.try_wait().ok().flatten().is_none() { let _ = self.0.kill(); } let _ = self.0.wait(); } }
struct StopPid(PathBuf);
impl Drop for StopPid { fn drop(&mut self) {
    if let Some(pid) = fs::read_to_string(&self.0).ok().and_then(|s| s.parse::<u32>().ok()) {
        if pid != std::process::id() && KillPidProbe.pid_alive(pid) { let _ = Command::new("kill").args(["-KILL", &pid.to_string()]).status(); }
    }
} }
