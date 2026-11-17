//! S-136/S-148/S-155: actual CLI undo/resolve/status/log/save/join -> real Join, Oplog, Snapshot, Gitio and Folders.
//! Each parent owns World's disposable store/worktrees/remote. Expected state is captured BEFORE final resolve;
//! expected folder_tree is built by raw Git from literal resolved contents, not Join's composition code.
//! Real post-undo faults and SIGKILL occur after the selected ref transaction, without clocks or fake kit answers.
use super::fixture::{assert_one_undo, done, index_at, observe, ours, World};
use crate::{json::Report, support::{actual_git, raw, script, shim_path, REQUEST, ZONE}};
use gitio::Oid;
use oplog::{JsonValue, KillPidProbe, LockName, OpPhase, PidProbe, ValuedRecord};
use std::{fs, os::unix::process::ExitStatusExt, path::PathBuf, process::{Command, Stdio}};
use testkit::{run_with_kill_gate, write_git_kill_shim};

const CHOSEN: &[u8] = b"chosen resolution\n";
const OUTSIDE: &[u8] = b"owner's unrelated edit\n";
const BAD: &[u8] = b"not a paused join\n";

struct Resolved { w: World, old: Oid, new: Oid, paused: Vec<u8>, target: ValuedRecord, folder_tree: Oid }
impl Resolved {
    fn new() -> Self {
        let w = World::new(); let old = w.s.f.save_topic();
        // Only the conflicted path changes upstream: zone-only additions become untracked after undo resets
        // the index, introducing a separate folder-move refusal rather than this clean restore round trip.
        let zone = w.s.f.commit(&w.s.f.base, &[("src/file", b"zone conflict\n")], "zone conflict\n");
        raw(&w.s.f.git, &["update-ref", ZONE, zone.as_str()]);
        ours(&w, &["join"], 2, "conflicted"); ours(&w, &["resolve"], 2, "files-written");
        w.s.f.write("src/file", CHOSEN); w.s.f.write("sub/keep", OUTSIDE);
        let paused = fs::read(state(&w)).unwrap();
        let literal = w.s.f.commit(&old, &[("src/file", CHOSEN)], "literal expected folder tree\n");
        let folder_tree = w.s.f.git.commit_info(&literal).unwrap().tree;
        ours(&w, &["resolve"], 0, "rebased"); let new = w.s.f.head(); assert_ne!(old, new);
        assert_eq!(w.s.f.git.commit_info(&new).unwrap().parents, vec![zone]);
        let mut resolved = resolve_records(&w); assert_eq!(resolved.len(), 1); let target = resolved.remove(0);
        assert_eq!(target.record.moves, vec![oplog::RefMove { refname: REQUEST.into(), old: Some(old.clone()), new: new.clone() }]);
        assert_eq!(target.values, vec![("folder_tree".into(), folder_tree.to_string()),
            ("join_state".into(), std::str::from_utf8(&paused).unwrap().strip_suffix('\n').unwrap().to_string())]);
        assert!(!state(&w).exists()); assert_eq!(fs::read(w.s.f.request.join("src/file")).unwrap(), CHOSEN);
        assert_eq!(fs::read(w.s.f.request.join("sub/keep")).unwrap(), OUTSIDE);
        Self { w, old, new, paused, target, folder_tree }
    }
    fn assert_report(&self, r: &Report) -> String {
        let op = assert_one_undo(&self.w, &self.target.record, &self.new, &self.old);
        assert_eq!(r.text("undone_op"), self.target.record.op_id); assert_eq!(r.text("undo_op"), op.op_id);
        assert_eq!(r.text("next"), "mr resolve"); assert_eq!(r.text("paused_commit"), self.old.as_str());
        assert_eq!(r.strings("conflicted"), &["src/file"]); self.assert_restored(); op.op_id
    }
    fn assert_restored(&self) {
        assert_eq!(self.w.s.f.head(), self.old); assert_eq!(fs::read(state(&self.w)).unwrap(), self.paused);
        index_at(&self.w, &self.w.s.f.request, &self.old);
        assert_eq!(fs::read(self.w.s.f.request.join("src/file")).unwrap(), CHOSEN);
        assert_eq!(fs::read(self.w.s.f.request.join("sub/keep")).unwrap(), OUTSIDE);
        let status = self.w.s.f.status();
        assert!(status.is_null("stalled_from") && status.is_null("stalled_to"));
        assert!(!self.w.s.f.lock_path().exists());
    }
    fn undo(&self) -> String { self.assert_report(&ours(&self.w, &["undo"], 0, "undone")) }
}
fn state(w: &World) -> PathBuf { w.s.f.local().git_dir(&w.s.f.request).unwrap().join("mortar-join.json") }
fn index_bytes(w: &World) -> Vec<u8> { fs::read(w.s.f.local().git_dir(&w.s.f.request).unwrap().join("index")).unwrap() }
fn resolve_records(w: &World) -> Vec<ValuedRecord> {
    oplog::read_valued_log(&w.s.f.common).unwrap().records.into_iter()
        .filter(|v| v.record.kind == "join-rebase" && v.record.phase == OpPhase::Done && v.record.actor == "core/task").collect()
}

#[test]
fn binary_resolve_undo_restores_exact_pause_status_has_no_stall_repeat_is_inert_and_resolve_reuses_folder_tree() {
    let s = Resolved::new(); let files = s.w.folder_state(&s.w.s.f.request).1;
    let undo = s.undo(); assert_eq!(s.w.folder_state(&s.w.s.f.request).1, files);
    let before = observe(&s.w); let index = index_bytes(&s.w);
    let status = ours(&s.w, &["status"], 0, "paused");
    assert_eq!(status.text("next"), "mr resolve"); assert_eq!(status.text("paused_commit"), s.old.as_str());
    assert_eq!(status.strings("paused_files"), &["src/file"]);
    assert!(status.is_null("stalled_from") && status.is_null("stalled_to"));
    assert_eq!(observe(&s.w), before); assert_eq!(index_bytes(&s.w), index); s.assert_restored();
    let repeat = ours(&s.w, &["undo"], 0, "already-done");
    assert_eq!(s.assert_report(&repeat), undo); assert_eq!(observe(&s.w), before);
    ours(&s.w, &["resolve"], 0, "rebased");
    let records = resolve_records(&s.w); assert_eq!(records.len(), 2);
    assert_eq!(records[1].values, s.target.values);
    assert_eq!(records[1].values[0], ("folder_tree".into(), s.folder_tree.to_string()));
    assert_eq!(s.w.s.f.git.commit_info(&s.w.s.f.head()).unwrap().tree, s.w.s.f.git.commit_info(&s.new).unwrap().tree);
    assert!(!state(&s.w).exists()); assert_eq!(s.w.folder_state(&s.w.s.f.request).1, files);
    assert_eq!(done(&s.w, "undo").len(), 1);
}

#[test]
fn binary_undo_precheck_keeps_another_real_paused_join_before_snapshot_ref_or_index_changes() {
    let s = Resolved::new(); let other_ref = "refs/heads/req/core/other";
    let other = s.w.s.f.commit(&s.w.s.f.base, &[("src/file", b"other request\n")], "other request\n");
    raw(&s.w.s.f.git, &["update-ref", other_ref, other.as_str()]);
    let answer = oplog::with_lock(&s.w.s.f.git, &s.w.s.f.common, &LockName::Join("core".into()), "other-pause", &KillPidProbe, || {
        oplog::recover(&s.w.s.f.git, &s.w.s.f.common).unwrap();
        join::join(&s.w.s.f.git, &s.w.s.f.request, "core", other_ref, "core/other")
    }).unwrap();
    assert!(matches!(answer, join::JoinAnswer::Conflicted { commit, files, .. }
        if commit == other && files == vec![PathBuf::from("src/file")]));
    let paused = fs::read(state(&s.w)).unwrap(); let before = observe(&s.w); let index = index_bytes(&s.w);
    let r = ours(&s.w, &["undo"], 1, "refused");
    assert_eq!(r.strings("lines"), &[format!("{} holds another paused join, which undo never overwrites", state(&s.w).display()), "nothing was changed".into()]);
    assert_eq!(r.text("next"), "mr status");
    assert_eq!(observe(&s.w), before); assert_eq!(index_bytes(&s.w), index);
    assert_eq!(fs::read(state(&s.w)).unwrap(), paused); assert_eq!(s.w.s.f.head(), s.new);
    assert!(done(&s.w, "undo").is_empty());
}

#[test]
fn binary_undo_precheck_keeps_corrupt_state_and_refuses_before_snapshot_or_any_undo() {
    let s = Resolved::new(); fs::write(state(&s.w), BAD).unwrap();
    let before = observe(&s.w); let index = index_bytes(&s.w);
    let r = ours(&s.w, &["undo"], 1, "refused");
    assert_eq!(r.strings("lines").len(), 3);
    assert!(r.strings("lines")[0].starts_with(&format!("{} is not a paused-join file mortar wrote: ", state(&s.w).display())));
    assert_eq!(&r.strings("lines")[1..], &["the file is kept", "nothing was changed"]);
    assert_eq!(r.text("next"), "mend or remove that file by hand, then mr undo");
    assert_eq!(observe(&s.w), before); assert_eq!(index_bytes(&s.w), index);
    assert_eq!(fs::read(state(&s.w)).unwrap(), BAD); assert_eq!(s.w.s.f.head(), s.new);
    assert!(done(&s.w, "undo").is_empty());
}

#[test]
fn binary_undo_refuses_a_real_resolve_record_in_the_legacy_format_without_recorded_join_state() {
    let s = Resolved::new(); let mut log = oplog::read_marked_log(&s.w.s.f.common).unwrap(); let mut removed = 0;
    for line in &mut log.records {
        if line.record.op_id == s.target.record.op_id {
            let n = line.values.len(); line.values.retain(|(name, _)| name != "join_state"); removed += n - line.values.len();
        }
    }
    assert_eq!(removed, 2, "one value removed from the genuine resolve's intent and done, retaining all other fields");
    // Public Oplog encoding makes an actual persisted legacy input; no regex or private Join codec.
    let text: String = log.records.iter().map(|m| format!("{}\n", oplog::encode_marked_record(&m.record, &m.values, m.remote.as_deref()))).collect();
    fs::write(oplog::oplog_path(&s.w.s.f.common), text).unwrap();
    assert_eq!(resolve_records(&s.w)[0].values, vec![("folder_tree".into(), s.folder_tree.to_string())]);
    let before = observe(&s.w); let index = index_bytes(&s.w);
    let r = ours(&s.w, &["undo"], 1, "refused");
    assert_eq!(r.strings("lines"), &[format!("op {} resolved a join before mortar recorded the paused join, so undo cannot put it back", s.target.record.op_id), "nothing was changed".into()]);
    assert_eq!(r.text("next"), "change the files, mr save, then mr join");
    assert_eq!(observe(&s.w), before); assert_eq!(index_bytes(&s.w), index);
    assert!(!state(&s.w).exists()); assert_eq!(s.w.s.f.head(), s.new); assert!(done(&s.w, "undo").is_empty());
}

fn member<'a>(value: &'a JsonValue, key: &str) -> &'a JsonValue {
    let JsonValue::Object(pairs) = value else { panic!("expected an object: {value:?}") };
    let mut found = pairs.iter().filter(|(name, _)| name == key);
    let (_, value) = found.next().unwrap_or_else(|| panic!("missing {key}: {pairs:?}")); assert!(found.next().is_none()); value
}
#[test]
fn binary_log_json_exposes_the_real_resolves_exact_recorded_join_state_on_intent_and_done_without_writing() {
    let s = Resolved::new(); let before = observe(&s.w);
    let output = s.w.s.f.app(&["--json", "log", "--ref", REQUEST]);
    assert_eq!(output.status.code(), Some(0), "{output:?}"); assert!(output.stderr.is_empty());
    let stdout = std::str::from_utf8(&output.stdout).unwrap();
    let body = stdout.strip_suffix("}\n").unwrap(); assert!(!body.contains('\n'));
    // records is log's last field, an array of raw op objects; its values have no JSON booleans.
    let (envelope, array) = body.rsplit_once(",\"records\":").unwrap();
    for field in ["\"verb\":\"log\"", "\"exit\":0", "\"ok\":true", "\"outcome\":\"listed\""] { assert!(envelope.contains(field), "{stdout}"); }
    let JsonValue::Array(rows) = oplog::parse_json(array).unwrap() else { panic!("records must be an array") };
    let selected: Vec<_> = rows.iter().filter(|row| member(row, "op_id") == &JsonValue::String(s.target.record.op_id.clone())).collect();
    assert_eq!(selected.len(), 2);
    let expected = JsonValue::Array(vec![
        JsonValue::Object(vec![("name".into(), JsonValue::String("folder_tree".into())), ("value".into(), JsonValue::String(s.folder_tree.to_string()))]),
        JsonValue::Object(vec![("name".into(), JsonValue::String("join_state".into())), ("value".into(), JsonValue::String(std::str::from_utf8(&s.paused).unwrap().strip_suffix('\n').unwrap().into()))]),
    ]);
    for (row, phase) in selected.iter().zip(["intent", "done"]) {
        assert_eq!(member(row, "phase"), &JsonValue::String(phase.into())); assert_eq!(member(row, "values"), &expected);
    }
    assert_eq!(observe(&s.w), before); assert!(!state(&s.w).exists());
}

#[test]
fn a_real_post_undo_restore_refusal_names_both_ops_exits_one_and_a_repeat_finishes_without_undoing_twice() {
    let s = Resolved::new(); let dir = s.w.s.f.common.join("restore-race-bin"); fs::create_dir(&dir).unwrap();
    let marker = s.w.s.f.common.join("restore-race-injected");
    script(&dir.join("git"), br#"#!/bin/sh
set -eu
if [ "${3-}" = update-ref ] && grep -q '"kind":"undo"' "$RESTORE_RIT_LOG"; then
  "$RESTORE_RIT_REAL_GIT" "$@"
  printf 'not a paused join\n' >"$RESTORE_RIT_STATE"
  printf 'after undo ref transaction\n' >"$RESTORE_RIT_MARKER"
  exit 0
fi
exec "$RESTORE_RIT_REAL_GIT" "$@"
"#);
    let files = s.w.folder_state(&s.w.s.f.request).1; let index = index_bytes(&s.w);
    let mut command = s.w.s.f.command(&s.w.s.f.request, &["--json", "undo"]);
    command.env("PATH", shim_path(&dir)).env("RESTORE_RIT_LOG", oplog::oplog_path(&s.w.s.f.common))
        .env("RESTORE_RIT_REAL_GIT", actual_git()).env("RESTORE_RIT_STATE", state(&s.w)).env("RESTORE_RIT_MARKER", &marker);
    let r = crate::json::check(command.output().unwrap(), "undo", 1, "refused");
    assert_eq!(fs::read(marker).unwrap(), b"after undo ref transaction\n");
    let undo = assert_one_undo(&s.w, &s.target.record, &s.new, &s.old);
    assert_eq!(r.text("undone_op"), s.target.record.op_id); assert_eq!(r.text("undo_op"), undo.op_id);
    assert_eq!(r.strings("lines")[0], format!("op {} was undone by op {}; the paused join was not put back", s.target.record.op_id, undo.op_id));
    assert!(r.strings("lines")[1].starts_with(&format!("{} is not a paused-join file mortar wrote: ", state(&s.w).display())));
    assert_eq!(r.strings("lines")[2], "the file is kept"); assert_eq!(r.strings("lines").len(), 3);
    assert_eq!(r.text("next"), "mr undo"); assert_eq!(s.w.s.f.head(), s.old);
    assert_eq!(fs::read(state(&s.w)).unwrap(), BAD); assert_eq!(index_bytes(&s.w), index);
    assert_eq!(s.w.folder_state(&s.w.s.f.request).1, files);
    // This file was deliberately created by this test's shim, not by Join; removing it clears the injected cause.
    fs::remove_file(state(&s.w)).unwrap(); let log = s.w.s.f.log();
    let repeat = ours(&s.w, &["undo"], 0, "already-done"); assert_eq!(s.assert_report(&repeat), undo.op_id);
    assert_eq!(s.w.s.f.log(), log); assert_eq!(s.w.folder_state(&s.w.s.f.request).1, files);
}

#[test]
fn a_cli_killed_after_its_actual_undo_transaction_recovers_and_repeats_the_restore_once() {
    let s = Resolved::new(); let shim = s.w.s.f.common.join("restore-undo-kill.sh"); write_git_kill_shim(&shim).unwrap();
    let dir = s.w.s.f.common.join("restore-undo-kill-bin"); fs::create_dir(&dir).unwrap();
    script(&dir.join("git"), br#"#!/bin/sh
set -eu
if [ "${3-}" = update-ref ] && grep -q '"kind":"undo"' "$RESTORE_RIT_LOG"; then
  PATH="$RESTORE_RIT_REAL_PATH" exec "$RESTORE_RIT_SHIM" "$@"
fi
exec "$RESTORE_RIT_REAL_GIT" "$@"
"#);
    let template = s.w.s.f.command(&s.w.s.f.request, &["--json", "undo"]);
    let mut command = Command::new("/bin/sh");
    command.current_dir(&s.w.s.f.request).env_clear()
        .envs(template.get_envs().filter_map(|(key, value)| value.map(|value| (key, value))))
        .args(["-c", "export MORTAR_TEST_WORKER_PID=$$; exec \"$@\"", "undo-restore-worker", env!("CARGO_BIN_EXE_mortar"), "--json", "undo"])
        .env("PATH", shim_path(&dir)).env("RESTORE_RIT_REAL_PATH", std::env::var_os("PATH").unwrap())
        .env("RESTORE_RIT_SHIM", shim).env("RESTORE_RIT_LOG", oplog::oplog_path(&s.w.s.f.common))
        .env("RESTORE_RIT_REAL_GIT", actual_git()).env("MORTAR_TEST_CRASH", "after").stdin(Stdio::null());
    let files = s.w.folder_state(&s.w.s.f.request).1;
    let output = run_with_kill_gate(&mut command, &s.w.s.f.common).unwrap(); assert_eq!(output.status.signal(), Some(9), "{output:?}");
    let holder = oplog::decode_lock_record(&fs::read_to_string(s.w.s.f.lock_path()).unwrap()).unwrap();
    assert!(!KillPidProbe.pid_alive(holder.pid)); assert_eq!(s.w.s.f.head(), s.old); assert!(!state(&s.w).exists());
    let records = s.w.s.f.records(); let undone: Vec<_> = records.iter().filter(|r| r.kind == "undo").collect();
    assert_eq!(undone.len(), 1); assert_eq!(undone[0].phase, OpPhase::Intent);
    assert_eq!(undone[0].moves, vec![oplog::RefMove { refname: REQUEST.into(), old: Some(s.new.clone()), new: s.old.clone() }]);
    let repeat = ours(&s.w, &["undo"], 0, "already-done"); s.assert_report(&repeat);
    assert_eq!(s.w.folder_state(&s.w.s.f.request).1, files);
    assert_eq!(oplog::open_intent(&s.w.s.f.common, &undone[0].op_id).unwrap(), None);
    let before = observe(&s.w); s.assert_report(&ours(&s.w, &["undo"], 0, "already-done")); assert_eq!(observe(&s.w), before);
}

#[test]
fn save_after_a_resolve_undo_keeps_every_file_instead_of_finishing_a_plain_rebase_undo_move() {
    let s = Resolved::new(); s.undo(); let files = s.w.folder_state(&s.w.s.f.request).1;
    ours(&s.w, &["save", "Save resolution after undo"], 0, "saved");
    assert_eq!(s.w.folder_state(&s.w.s.f.request).1, files);
    assert_eq!(s.w.s.f.blob(&s.w.s.f.head(), "src/file"), CHOSEN); assert_eq!(done(&s.w, "undo").len(), 1);
}

#[test]
fn join_after_a_resolve_undo_keeps_every_file_instead_of_finishing_a_plain_rebase_undo_move() {
    let s = Resolved::new(); s.undo(); let files = s.w.folder_state(&s.w.s.f.request).1;
    let r = ours(&s.w, &["join"], 2, "conflicted"); assert_eq!(r.text("next"), "mr resolve");
    assert_eq!(s.w.folder_state(&s.w.s.f.request).1, files); assert_eq!(done(&s.w, "undo").len(), 1);
}
