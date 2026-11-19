use crate::{json::check, objects::set_zone, paused::Paused, support::*};
use crate::support::FormatterConfig;
use std::fs;

#[test]
fn rit_no_pause_is_exit_zero_with_real_pre_act_snapshot_and_inert_repeat() {
    let f = Fixture::new(); f.write("src/file", b"unsaved work without a pause\n");
    let old = f.head(); let start = f.records().len();
    let report = f.report(&["--json", "resolve"], "resolve", 0, "nothing-to-resolve");
    assert_eq!(report.text("next"), "mr join");
    assert_eq!(report.strings("lines"), &["the snapshot ref moved as the work was saved first", "no join is paused in this folder; nothing to resolve"]);
    assert_eq!(f.head(), old); assert_eq!(fs::read(f.request.join("src/file")).unwrap(), b"unsaved work without a pause\n");
    let snap = f.git.read_ref(SNAP).unwrap().unwrap();
    assert_eq!(f.blob(&snap, "src/file"), b"unsaved work without a pause\n");
    let added = f.records()[start..].to_vec(); assert_eq!(added.len(), 2);
    assert!(added.iter().all(|r| r.kind == "snapshot" && r.actor == "core/task"));
    assert!(!f.lock_path().exists());
    assert!(join::paused_join(&f.local().git_dir(&f.request).unwrap()).unwrap().is_none());
    let before = f.observe(); f.report(&["--json", "resolve"], "resolve", 0, "nothing-to-resolve");
    assert_eq!(f.observe(), before); assert!(!f.lock_path().exists());
}

#[test]
fn rit_first_resolve_writes_exact_conflicted_bytes_after_snapshot_from_subdirectory_without_formatter() {
    let r = Paused::new(false); let f = &r.f;
    f.write("src/file", b"local work before marker overwrite\n"); f.write("sub/local", b"untracked work\n");
    let program = f.common.join("must-not-format.sh"); let trace = f.common.join("formatter-ran");
    script(&program, b"#!/bin/sh\nprintf 'formatter ran\n' >\"$1\"\n");
    let mut config = f.init_args.clone(); config.formatter = Some(FormatterConfig {
        command: vec![program.to_str().unwrap().into(), trace.to_str().unwrap().into()], timeout_seconds: 60 }); f.configure(&config);
    let expected = f.blob(&r.tree, "src/file");
    let conflict_blob = f.git.tree_entry(&r.tree, "src/file".as_ref()).unwrap().unwrap();
    let unrelated_index = raw(&f.local(), &["ls-files", "--stage", "-z", "--", "root.txt", "sub/keep"]);
    let start = f.records().len();
    let report = check(f.command(&f.request.join("src"), &["--json", "resolve"]).output().unwrap(), "resolve", 2, "files-written");
    assert_eq!(report.text("commit"), r.bad.as_str()); assert_eq!(report.text("request_ref"), REQUEST);
    assert_eq!(report.strings("files"), &["src/file"]); assert!(report.strings("skipped").is_empty()); assert!(report.strings("gitlinks").is_empty());
    assert_eq!(report.text("next"), "fix the files, then mr resolve");
    assert_eq!(fs::read(f.request.join("src/file")).unwrap(), expected);
    let text = String::from_utf8(expected).unwrap();
    assert!(text.contains("<<<<<<<") && text.contains("zone work\n") && text.contains("topic work\n") && text.contains(">>>>>>>"));
    let snap = f.git.read_ref(SNAP).unwrap().unwrap();
    assert_eq!(f.blob(&snap, "src/file"), b"local work before marker overwrite\n");
    assert_eq!(f.blob(&snap, "sub/local"), b"untracked work\n");
    assert_eq!(fs::read(f.request.join("sub/local")).unwrap(), b"untracked work\n");
    // Gitio write_blob_paths writes the named file AND its index entry; unrelated entries stay unchanged.
    assert_eq!(raw(&f.local(), &["ls-files", "--stage", "-z", "--", "src/file"]), format!("100644 {conflict_blob} 0\tsrc/file\0").as_bytes());
    assert_eq!(raw(&f.local(), &["ls-files", "--stage", "-z", "--", "root.txt", "sub/keep"]), unrelated_index);
    assert!(raw(&f.local(), &["ls-files", "--stage", "-z", "--", "sub/local"]).is_empty());
    let added = f.records()[start..].to_vec(); assert_eq!(added.len(), 2); assert!(added.iter().all(|v| v.kind == "snapshot"));
    assert!(fs::read_to_string(&r.state).unwrap().contains("\"files_written\":true"));
    assert!(!trace.exists(), "resolve must not format files that can hold conflict markers"); r.assert_unmoved();
}

#[test]
fn rit_marker_repeat_is_exit_two_naming_literal_lines_without_rewriting_work_or_paused_state() {
    let r = Paused::new(false); r.first(); let f = &r.f;
    let bytes = fs::read(f.request.join("src/file")).unwrap(); let state = r.state_bytes();
    // This fixture conflicts on its sole line: unlike a prefixed file, its marker lines are 1/3/5.
    let text = String::from_utf8(bytes.clone()).unwrap(); let lines: Vec<_> = text.lines().collect();
    assert!(lines[0].starts_with("<<<<<<<")); assert_eq!(lines[2], "======="); assert!(lines[4].starts_with(">>>>>>>"));
    let report = f.report(&["--json", "resolve"], "resolve", 2, "still-marked");
    assert!(report.lines().contains("src/file (lines 1, 3, 5)"), "{}", report.lines());
    assert_eq!(report.text("next"), "fix the files, then mr resolve");
    assert_eq!(fs::read(f.request.join("src/file")).unwrap(), bytes); assert_eq!(r.state_bytes(), state);
    let snap = f.git.read_ref(SNAP).unwrap().unwrap(); assert_eq!(f.blob(&snap, "src/file"), bytes);
    assert!(crate::objects::rebase_records(f).is_empty()); r.assert_unmoved();
}

#[test]
fn rit_markers_left_after_a_finished_move_name_the_snapshot_that_move_wrote() {
    // A rebase join leaves its join-rebase op as the request's last mover, so each resolve finishes it again (a no-op
    // folder move) after a snapshot; with markers written since, that snapshot is the only one that writes.
    let f = Fixture::new(); f.save_topic(); let zone1 = f.advance_zone(false);
    f.report(&["--json", "join"], "join", 0, "rebased"); let rebased = f.head();
    let zone2 = f.commit(&zone1, &[("src/file", b"zone work\n")], "zone advanced again\n"); set_zone(&f, &zone2);
    let r = Paused::start(f, &zone1, &rebased); r.first();
    let markers = fs::read(r.f.request.join("src/file")).unwrap();
    let start = r.f.records().len();
    let report = r.f.report(&["--json", "resolve"], "resolve", 2, "still-marked");
    let text = String::from_utf8(markers.clone()).unwrap();
    let marked: Vec<String> = text.lines().enumerate()
        .filter(|(_, line)| ["<<<<<<<", "=======", ">>>>>>>"].iter().any(|marker| line.starts_with(marker)))
        .map(|(at, _)| (at + 1).to_string()).collect();
    assert_eq!(report.strings("lines"), &[
        "conflict markers remain in these files; the snapshot ref moved as the work was saved first; no other commit \
         was made and no other ref moved:".to_string(),
        format!("  src/file (lines {})", marked.join(", ")),
    ]);
    let added = r.f.records()[start..].to_vec(); assert_eq!(added.len(), 2);
    assert!(added.iter().all(|v| v.kind == "snapshot"));
    let snap = r.f.git.read_ref(SNAP).unwrap().unwrap(); assert_eq!(r.f.blob(&snap, "src/file"), markers);
    r.assert_unmoved();
}
