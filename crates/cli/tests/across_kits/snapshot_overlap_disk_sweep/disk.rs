//! CLI run_disk -> real Folders disk/Gitio store and worktree measurements, with independent byte oracles.
use super::fixture::*;
use crate::support::{raw, write, REQUEST};
use oplog::{parse_json, JsonValue};
use std::path::Path;

#[test]
fn disk_lists_real_zone_and_request_bytes_counts_the_shared_store_once_and_never_sweeps_builds() {
    let w = World::new(); write(&w.s.f.request, "src/extra", b"request");
    write(&w.s.f.request, "target/debug/out", b"1234567890123"); write(&w.s.f.zone, "target/out", b"1234567");
    // The unrecorded main checkout must not contribute another folder, or have its target counted/swept.
    write(&w.s.f.git.repo, "target/out", b"main output is not a Mortar folder");
    let store = bytes_in_tree(&w.s.f.common.join("objects")); assert!(store > 0);
    let before = observe(&w); let r = app(&w, &["disk"], 0, "measured");
    let mut paths = r.strings("folders").to_vec(); paths.sort();
    let mut expected = vec![w.s.f.zone.display().to_string(), w.s.f.request.display().to_string(), w.b.display().to_string()]; expected.sort();
    assert_eq!(paths, expected);
    for line in [
        format!("{}: the zone folder of zone core; files 15 bytes, build 7 bytes", w.s.f.zone.display()),
        format!("{}: the zone folder of zone b; files 15 bytes, build 0 bytes", w.b.display()),
        format!("{}: the request folder of request task in zone core; files 22 bytes, build 13 bytes", w.s.f.request.display()),
    ] { assert!(r.strings("lines").contains(&line), "missing {line}: {r:?}"); }
    assert_eq!(r.number("store_bytes"), store);
    assert_eq!(r.strings("lines").iter().filter(|s| s.starts_with("object store:")).cloned().collect::<Vec<_>>(), vec![format!("object store: {store} bytes, counted once")]);
    assert_eq!(r.strings("refs"), &["refs/heads 2", "refs/heads/req 1", "refs/heads/zone 2", "refs/remotes/origin 1"]);
    assert_eq!(r.number("sweep_bytes"), 0, "sweep policy: no stale refs and no implicit build sweep");
    assert!(r.strings("lines").contains(&"an mr sweep with no --idle-days frees up to 0 bytes".into()));
    assert_eq!(r.text("next"), "mr sweep --idle-days <n> --actor <name>"); assert_eq!(observe(&w), before);
    // The same object store, not a per-cwd or per-folder sum, when called from the request instead of main.
    let again = run(&w, &w.s.f.request, &["disk"], 0, "measured");
    assert_eq!(again.number("store_bytes"), store); assert_eq!(again.strings("lines"), r.strings("lines"));
    assert_eq!(observe(&w), before);
}

fn disk_holds_report(w: &World) -> JsonValue {
    let output = w.s.f.command(&w.s.f.git.repo, &["--json", "--actor", "snapshot-overlap-disk-sweep", "disk"]).output().unwrap();
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    parse_json(std::str::from_utf8(&output.stdout).unwrap()).unwrap()
}

fn member<'a>(object: &'a [(String, JsonValue)], key: &str) -> &'a JsonValue {
    let matching: Vec<_> = object.iter().filter(|(name, _)| name == key).collect();
    assert_eq!(matching.len(), 1, "expected one {key}: {object:?}");
    &matching[0].1
}

fn assert_disk_hold(report: &JsonValue, path: &Path, kind: &str, text: &str, counts: (u64, u64), joined: JsonValue, no_joined_answer: JsonValue) {
    let JsonValue::Object(fields) = report else { panic!("not an object: {report:?}") };
    let JsonValue::Array(lines) = member(fields, "lines") else { panic!("lines is not an array") };
    let line = JsonValue::String(format!("{}: {text}", path.display()));
    assert!(lines.contains(&line), "missing line {line:?}: {lines:?}");
    let at = lines.iter().position(|item| item == &line).unwrap();
    assert!(at > 0);
    let JsonValue::String(previous) = &lines[at - 1] else { panic!("previous line is not text") };
    assert!(previous.starts_with(&format!("{}: {kind}; files ", path.display())), "{previous}");
    assert!(previous.contains(" bytes, build ") && previous.ends_with(" bytes"), "{previous}");
    assert!(lines.iter().all(|line| !matches!(line, JsonValue::String(s) if s.to_lowercase().contains("safe"))));
    assert_eq!(fields.last().map(|(name, _)| name.as_str()), Some("holds"));
    let JsonValue::Array(folders) = member(fields, "folders") else { panic!("folders is not an array") };
    let JsonValue::Array(holds) = member(fields, "holds") else { panic!("holds is not an array") };
    assert_eq!(holds.len(), folders.len());
    for (folder, hold) in folders.iter().zip(holds) {
        let JsonValue::Object(object) = hold else { panic!("hold is not an object: {hold:?}") };
        assert_eq!(member(object, "path"), folder);
    }
    let index = folders.iter().position(|folder| folder == &JsonValue::String(path.display().to_string())).unwrap();
    let JsonValue::Object(object) = &holds[index] else { unreachable!() };
    assert_eq!(object.iter().map(|(name, _)| name.as_str()).collect::<Vec<_>>(),
        ["path", "tracked", "untracked", "joined", "no_joined_answer"]);
    assert_eq!(member(object, "tracked"), &JsonValue::Number(counts.0));
    assert_eq!(member(object, "untracked"), &JsonValue::Number(counts.1));
    assert_eq!(member(object, "joined"), &joined);
    assert_eq!(member(object, "no_joined_answer"), &no_joined_answer);
}

#[test]
fn disk_holds_dirty_request_counts_without_writing() {
    let w = World::new();
    write(&w.s.f.request, "src/file", b"edited\n");
    write(&w.s.f.request, "src/new-one", b"one\n");
    write(&w.s.f.request, "src/new-two", b"two\n");
    assert_eq!(raw(&w.git(&w.s.f.request), &["status", "--porcelain", "--untracked-files=all"]),
        b" M src/file\n?? src/new-one\n?? src/new-two\n");
    let before = observe(&w);
    let objects = bytes_in_tree(&w.s.f.common.join("objects"));
    let report = disk_holds_report(&w);
    assert_eq!(observe(&w), before);
    assert_eq!(bytes_in_tree(&w.s.f.common.join("objects")), objects);
    assert_disk_hold(&report, &w.s.f.request, "the request folder of request task in zone core",
        "unsaved 1 tracked, 2 untracked; not joined to zone core", (1, 2), JsonValue::Bool(false), JsonValue::Null);
}

#[test]
fn disk_holds_clean_saved_and_joined_request() {
    let w = World::new();
    write(&w.s.f.request, "src/file", b"saved work\n");
    saved(&w, &w.s.f.request, "Save topic");
    w.s.f.report(&["--json", "join"], "join", 0, "joined");
    assert_eq!(raw(&w.git(&w.s.f.request), &["status", "--porcelain", "--untracked-files=all"]), b"");
    let report = disk_holds_report(&w);
    assert_disk_hold(&report, &w.s.f.request, "the request folder of request task in zone core",
        "unsaved 0 tracked, 0 untracked; joined to zone core", (0, 0), JsonValue::Bool(true), JsonValue::Null);
}

#[test]
fn disk_holds_zone_counts_without_a_joined_answer() {
    let w = World::new();
    write(&w.s.f.zone, "src/file", b"zone edit\n");
    write(&w.s.f.zone, "src/new-one", b"one\n");
    write(&w.s.f.zone, "src/new-two", b"two\n");
    assert_eq!(raw(&w.git(&w.s.f.zone), &["status", "--porcelain", "--untracked-files=all"]),
        b" M src/file\n?? src/new-one\n?? src/new-two\n");
    let report = disk_holds_report(&w);
    assert_disk_hold(&report, &w.s.f.zone, "the zone folder of zone core",
        "unsaved 1 tracked, 2 untracked; a zone folder has no joined answer", (1, 2), JsonValue::Null,
        JsonValue::String("a zone folder".into()));
}

#[test]
fn disk_answers_every_folder_when_one_joined_read_fails() {
    let w = World::new();
    let healthy = w.request("b", "healthy");
    raw(&w.s.f.git, &["update-ref", "-d", REQUEST]);
    assert!(w.s.f.git.read_ref(REQUEST).unwrap().is_none());
    let before = observe(&w);
    let objects = object_bytes(&w.s.f.common.join("objects"));
    let output = w.s.f.command(&w.s.f.git.repo, &["--json", "--actor", "snapshot-overlap-disk-sweep", "disk"]).output().unwrap();
    assert_eq!(observe(&w), before);
    assert_eq!(object_bytes(&w.s.f.common.join("objects")), objects);
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    let report = parse_json(std::str::from_utf8(&output.stdout).unwrap()).unwrap();
    let JsonValue::Object(fields) = &report else { panic!("not an object: {report:?}") };
    assert_eq!(member(fields, "verb"), &JsonValue::String("disk".into()));
    assert_eq!(member(fields, "outcome"), &JsonValue::String("measured".into()));
    let reason = format!("the branch {REQUEST} does not exist");
    assert_disk_hold(&report, &w.s.f.request, "the request folder of request task in zone core",
        &format!("unsaved 3 tracked, 0 untracked; joined unknown: {reason}"), (3, 0), JsonValue::Null,
        JsonValue::String(reason));
    assert_disk_hold(&report, &healthy, "the request folder of request healthy in zone b",
        "unsaved 0 tracked, 0 untracked; not joined to zone b", (0, 0), JsonValue::Bool(false), JsonValue::Null);
}

fn assert_unknown_reason(report: &JsonValue, path: &Path, kind: &str, counts: (u64, u64), reason: &str) {
    let JsonValue::Object(fields) = report else { panic!("not an object: {report:?}") };
    assert_eq!(member(fields, "verb"), &JsonValue::String("disk".into()));
    assert_eq!(member(fields, "outcome"), &JsonValue::String("measured".into()));
    assert_disk_hold(report, path, kind,
        &format!("unsaved {} tracked, {} untracked; joined unknown: {reason}", counts.0, counts.1),
        counts, JsonValue::Null, JsonValue::String(reason.into()));
    for git_text in ["fatal:", "git rev-parse", "git cat-file", "git show-ref"] {
        assert!(!reason.contains(git_text), "git wording leaked: {reason}");
    }
}

#[test]
fn disk_unknown_reason_for_unbranchable_request_record_is_cli_text() {
    let w = World::new();
    let dir = w.s.f.git.git_dir(&w.s.f.request).unwrap();
    let path = folders::record::record_path(&dir);
    let original = std::fs::read_to_string(&path).unwrap();
    assert_eq!(original.matches("\"request\":\"task\"").count(), 1);
    let changed = original.replace("\"request\":\"task\"", "\"request\":\"has space\"");
    assert_eq!(folders::record::decode_folder_record(&changed).unwrap().request.as_deref(), Some("has space"));
    std::fs::write(path, changed).unwrap();
    let listed = folders::disk(&w.s.f.git, 0, &[], &[], &[]).unwrap();
    assert!(listed.folders.iter().any(|f| f.path == w.s.f.request && f.record.request.as_deref() == Some("has space")));
    let before = observe(&w);
    let report = disk_holds_report(&w);
    assert_eq!(observe(&w), before);
    assert_unknown_reason(&report, &w.s.f.request, "the request folder of request has space in zone core", (0, 0),
        "request has space in zone core is not a name a branch can carry");
}

fn object_bytes(path: &Path) -> Vec<(std::path::PathBuf, Vec<u8>)> {
    fn collect(root: &Path, path: &Path, out: &mut Vec<(std::path::PathBuf, Vec<u8>)>) {
        for entry in std::fs::read_dir(path).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() { collect(root, &path, out); }
            else { out.push((path.strip_prefix(root).unwrap().into(), std::fs::read(&path).unwrap())); }
        }
    }
    let mut out = Vec::new();
    collect(path, path, &mut out);
    out.sort();
    out
}
