//! An attempt folder's unborn HEAD through the mortar binary: status refuses it with the try branch as next, need is
//! refused as an attempt folder before HEAD is read, disk lists its joined answer as unknown with no next of its own for that folder;
//! each run as a child in a throwaway repository with a real Git worktree.
use crate::{json::check, observe::before, support::{raw, setup, Fixture}};
use oplog::{parse_json, JsonValue};
use std::path::PathBuf;

fn unborn_attempt(f: &Fixture) -> PathBuf {
    let dir = f.root.join("attempts");
    oplog::recover(&f.git, &f.common).unwrap();
    folders::make_attempts(&setup(&f.git, &dir), "core", "task", &["src".into()], 1).unwrap();
    let attempt = dir.join("try-1");
    let git = gitio::Git { repo: attempt.clone(), ..f.git.clone() };
    raw(&git, &["checkout", "--orphan", "unborn-test"]);
    assert!(f.git.read_ref("refs/heads/try/core/task/1").unwrap().is_some());
    attempt
}

#[test]
fn attempt_unborn_head_status() {
    let f = Fixture::new();
    let attempt = unborn_attempt(&f);
    let old = before(&f, &attempt);
    let report = check(f.command(&attempt, &["--json", "status"]).output().unwrap(), "status", 1, "refused");
    assert_eq!(before(&f, &attempt), old);
    assert_eq!(report.text("owner"), "the folder's HEAD");
    assert_eq!(report.strings("lines"), &[format!("HEAD of the folder {} has no commit", attempt.display())]);
    assert_eq!(report.text("next"), "check out the branch try/core/task/1 in the attempt's folder, then repeat the same command");
}

#[test]
fn attempt_unborn_head_need() {
    let f = Fixture::new();
    let attempt = unborn_attempt(&f);
    let old = before(&f, &attempt);
    let report = check(f.command(&attempt, &["--json", "need", "src"]).output().unwrap(), "need", 1, "refused");
    assert_eq!(before(&f, &attempt), old);
    assert_eq!(report.text("next"), "keep or drop the attempts, then mr join in the request's folder");
}

#[test]
fn attempt_unborn_head_disk() {
    let f = Fixture::new();
    let attempt = unborn_attempt(&f);
    let output = f.command(&f.git.repo, &["--json", "--actor", "harness", "disk"]).output().unwrap();
    let stdout = std::str::from_utf8(&output.stdout).unwrap();
    assert!(!stdout.contains("request's branch"));
    let JsonValue::Object(fields) = parse_json(stdout).unwrap() else { panic!("disk report is not an object") };
    let JsonValue::Array(holds) = &fields.iter().find(|(key, _)| key == "holds").unwrap().1 else { panic!("holds is not an array") };
    let hold = holds.iter().find_map(|hold| match hold {
        JsonValue::Object(fields) if fields.iter().any(|(key, value)| key == "path" && *value == JsonValue::String(attempt.display().to_string())) => Some(fields),
        _ => None,
    }).unwrap();
    let reason = format!("HEAD of the folder {} has no commit", attempt.display());
    assert!(hold.contains(&("joined".into(), JsonValue::Null)));
    assert!(hold.contains(&("no_joined_answer".into(), JsonValue::String(reason.clone()))));
    let report = check(output, "disk", 0, "measured");
    assert!(report.strings("lines").iter().any(|line| line.starts_with(&format!("{}: ", attempt.display())) && line.ends_with(&format!("joined unknown: {reason}"))));
}
