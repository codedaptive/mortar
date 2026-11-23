//! Actual mortar resolve SIGKILL after durable final op/state removal -> next mortar join, through
//! CLI read_pending_move/finish_pending_move, Oplog, Snapshot, Join and Folders/Gitio. No timing oracle.
use crate::{objects::{assert_one_rebase, tree}, paused::Paused, support::*};
use oplog::{KillPidProbe, OpPhase, PidProbe};
use std::{fs::{self, File, OpenOptions}, io::{BufRead, BufReader, Write}, os::unix::process::ExitStatusExt,
    path::PathBuf, process::{Child, Command}, sync::mpsc, thread, time::Duration};

#[test]
fn killed_resolve_after_final_done_op_and_removed_state_is_finished_from_folder_tree_by_next_join_once() {
    let r = Paused::new(false); r.first(); let f = &r.f;
    f.write("src/file", b"chosen resolution\n");
    let expected_tree = tree(f, &r.old, &[("src/file", b"chosen resolution\n")], &[], &[]);
    let index_before = raw(&f.local(), &["ls-files", "--stage", "-z"]);
    let state_before = fs::read_to_string(&r.state).unwrap();
    let state_before = state_before.strip_suffix('\n').expect("state file's trailing newline").to_string();
    let dir = f.common.join("resolve-git-shim"); fs::create_dir(&dir).unwrap();
    let event = f.common.join("resolve-before-move"); let release = f.common.join("resolve-release");
    let guard = GateGuard::new(vec![event.clone(), release.clone()]);
    script(&dir.join("git"), br#"#!/bin/sh
set -eu
if [ "${3-}" = read-tree ] && [ "${4-}" = --no-sparse-checkout ] &&
   mkdir "$MORTAR_VALIDATION_GATE_ONCE" 2>/dev/null; then
  printf 'BEFORE_MOVE\n' >"$MORTAR_VALIDATION_GATE_EVENT"
  IFS= read -r signal <"$MORTAR_VALIDATION_GATE_RELEASE"
  [ "$signal" = release ] || exit 97
fi
exec "$MORTAR_VALIDATION_REAL_GIT" "$@"
"#);
    let mut command = f.command(&f.request, &["--json", "resolve"]);
    command.env("PATH", shim_path(&dir)).env("MORTAR_VALIDATION_REAL_GIT", actual_git())
        .env("MORTAR_VALIDATION_GATE_ONCE", f.common.join("resolve-gated-once"))
        .env("MORTAR_VALIDATION_GATE_EVENT", &event).env("MORTAR_VALIDATION_GATE_RELEASE", &release);
    let mut child = Reap(command.spawn().unwrap());
    let mut line = String::new(); BufReader::new(File::open(&event).unwrap()).read_line(&mut line).unwrap();
    assert_eq!(line, "BEFORE_MOVE\n", "the gate is the actual folder-move check, not a synthetic intent");
    let pid = child.0.id();
    let holder = oplog::decode_lock_record(&fs::read_to_string(f.lock_path()).unwrap()).unwrap();
    assert_eq!(holder.pid, pid);
    let new = f.head(); assert_ne!(new, r.old); assert!(!r.state.exists());
    assert_eq!(f.git.commit_info(&new).unwrap().parents, vec![r.zone.clone()]);
    assert_eq!(f.blob(&new, "src/file"), b"chosen resolution\n");
    assert_eq!(f.blob(&new, "src/upstream"), b"advanced zone work\n");
    assert!(!f.request.join("src/upstream").exists());
    assert_eq!(raw(&f.local(), &["ls-files", "--stage", "-z"]), index_before);
    assert_one_rebase(f, &r.old, &new);
    let valued = oplog::read_valued_log(&f.common).unwrap();
    let done = valued.records.iter().find(|v| v.record.kind == "join-rebase" && v.record.phase == OpPhase::Done).unwrap();
    let folder_value = ("folder_tree".to_string(), expected_tree.as_str().to_string());
    // Pin both complete values in order, including the pre-resolve state text, then accept the base_before/base_after
    // that WO-join-base-in-ops (totppouw) appends; G10 (WO-cli-g10-join-boundary) tightens this to the four-value form only.
    assert_values_then_bases(&done.values, &[folder_value, ("join_state".to_string(), state_before)]);
    assert_eq!(oplog::open_intent(&f.common, &done.record.op_id).unwrap(), None);
    let snap = f.git.read_ref(SNAP).unwrap().unwrap();
    assert_eq!(f.blob(&snap, "src/file"), b"chosen resolution\n");
    child.0.kill().unwrap(); assert_eq!(child.0.wait().unwrap().signal(), Some(9));
    assert!(!KillPidProbe.pid_alive(pid), "wait/reap the lock holder BEFORE releasing the git shim");
    release_gate(&release, b"release\n");
    // Status need not call a marker-written index stalled; the next binary join must consume this tree.
    let before_read = f.observe(); let pending = f.last_move();
    assert_eq!(pending.record.op_id, done.record.op_id);
    let movement = pending.record.moves.iter().find(|m| m.refname == REQUEST).unwrap();
    assert_eq!(movement.old.as_ref(), Some(&r.old)); assert_eq!(movement.new, new);
    assert_eq!(pending.values[0], ("folder_tree".into(), expected_tree.to_string()));
    assert_eq!(f.observe(), before_read);
    let report = f.report(&["--json", "join"], "join", 0, "joined");
    assert_eq!(report.text("finished_move_from"), r.old.as_str());
    assert_eq!(report.text("finished_move_to"), new.as_str());
    assert!(!report.flag("saved_written"), "catch-up must precede saving the old folder as request work");
    assert_eq!(f.head(), new); assert_eq!(f.local().rev_parse("HEAD").unwrap(), Some(new.clone()));
    // The join after catch-up then fast-forwards the zone, unlike resolve itself.
    assert_eq!(f.git.read_ref(ZONE).unwrap(), Some(new.clone()));
    assert_eq!(fs::read(f.request.join("src/file")).unwrap(), b"chosen resolution\n");
    assert_eq!(fs::read(f.request.join("src/upstream")).unwrap(), b"advanced zone work\n");
    assert_eq!(raw(&f.local(), &["write-tree"]), format!("{}\n", f.git.commit_info(&new).unwrap().tree).as_bytes());
    assert!(raw(&f.local(), &["status", "--porcelain"]).is_empty());
    assert_one_rebase(f, &r.old, &new); assert!(!r.state.exists()); assert!(!f.lock_path().exists());
    let before_repeat = f.observe();
    let repeat = f.report(&["--json", "join"], "join", 0, "joined");
    assert_eq!(repeat.text("finished_move_to"), new.as_str()); assert!(!repeat.flag("saved_written"));
    assert_eq!(f.observe(), before_repeat); assert!(!f.lock_path().exists());
    drop(guard);
}

fn release_gate(path: &PathBuf, bytes: &[u8]) {
    if let Ok(mut file) = OpenOptions::new().read(true).write(true).open(path) { let _ = file.write_all(bytes); }
}
// Emergency escape only, not a runtime deadline or a performance assertion. The oracle is the FIFO event
// and the persisted Done/state/files; an escaped gate never supplies the required BEFORE_MOVE event.
struct GateGuard { cancel: Option<mpsc::Sender<()>>, worker: Option<thread::JoinHandle<()>>, paths: Vec<PathBuf> }
impl GateGuard {
    fn new(paths: Vec<PathBuf>) -> Self {
        assert!(Command::new("mkfifo").args(&paths).status().unwrap().success());
        let (send, recv) = mpsc::channel(); let release = paths.clone();
        let worker = thread::spawn(move || { if recv.recv_timeout(Duration::from_secs(120)).is_err() {
            for path in release { release_gate(&path, b"GATE_NOT_REACHED\n"); }
        } });
        Self { cancel: Some(send), worker: Some(worker), paths }
    }
}
impl Drop for GateGuard { fn drop(&mut self) {
    for path in &self.paths { release_gate(path, b"STOP\n"); }
    if let Some(cancel) = self.cancel.take() { let _ = cancel.send(()); }
    if let Some(worker) = self.worker.take() { let _ = worker.join(); }
} }
struct Reap(Child);
impl Drop for Reap { fn drop(&mut self) {
    if self.0.try_wait().ok().flatten().is_none() { let _ = self.0.kill(); }
    let _ = self.0.wait();
} }
