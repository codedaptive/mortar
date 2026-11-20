use std::{fs, path::Path, process::Command};
use crate::{json::{self, Value}, support::OwnDir};
use super::run_rereview_tests::{repo, commit_on, squash_in_setup};

pub(super) fn git_count(cwd: &Path, home: &Path, env: &[(std::ffi::OsString, std::ffi::OsString)], args: &[&str]) -> usize {
    let out = Command::new("git").arg("-C").arg(cwd).args(args).env_clear().envs(env.iter().cloned())
        .env("PATH", std::env::var_os("PATH").unwrap()).env("HOME", home).env("TMPDIR", home)
        .env("GIT_CONFIG_GLOBAL", "/dev/null").env("GIT_CONFIG_NOSYSTEM", "1")
        .env("LC_ALL", "C").env("GIT_TERMINAL_PROMPT", "0").env("GIT_OPTIONAL_LOCKS", "0").output().unwrap();
    assert!(out.status.success(), "{out:?}");
    out.stdout.iter().filter(|b| **b & 0xc0 != 0x80).count()
}

#[test]
fn savings_rereview_real_diff_and_log() {
    let (_repo, root) = repo();
    let home = OwnDir::new();
    commit_on(&root, "zone/a", "src/a/x", "first edit\n");
    let reviewed = squash_in_setup(&root, "a");
    let patch: String = (0..400).map(|i| format!("rereview changed line {i} é\n")).collect();
    commit_on(&root, "zone/a", "src/a/x", &patch);
    let new = squash_in_setup(&root, "a");
    let b = git_count(&root, &home.0, &[], &["--no-pager", "-c", "color.ui=never", "diff", "--no-color", "--no-ext-diff", reviewed.commit.as_str(), new.commit.as_str(), "--"]);
    let run = |json: bool| {
        let mut c = Command::new(env!("CARGO_BIN_EXE_mortar"));
        c.current_dir(&root).env_clear().env("PATH", std::env::var_os("PATH").unwrap())
            .env("HOME", &home.0).env("TMPDIR", &home.0).env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("GIT_CONFIG_NOSYSTEM", "1").env("CODETEAM_SESSION", "rereview-test");
        if json { c.arg("--json"); }
        c.args(["--actor", "accept", "rereview", reviewed.commit.as_str(), new.commit.as_str()]).output().unwrap()
    };
    let output = run(false);
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    let text = std::str::from_utf8(&output.stdout).unwrap();
    assert!(text.starts_with("[mr] tokens saved ≈ "), "{text}");
    let (line, body) = text.split_once("\n\n").unwrap();
    assert!(line.contains("git diff <reviewed> <new>"), "{line}");
    assert!(b > body.chars().count());
    let out = run(true);
    let raw = std::str::from_utf8(&out.stdout).unwrap();
    assert!(raw.ends_with(&format!("\"savings\":{{\"baseline\":\"git diff <reviewed> <new>\",\"baselineChars\":{b},\"outputChars\":{}}}}}\n", body.chars().count())), "{raw}");
    let report = json::read(out, "rereview", 0);
    assert!(report.object("savings").contains(&("baselineChars".into(), Value::Number(b as u64))));
    let log = fs::read_to_string(home.0.join(".local/state/mortar/savings.jsonl")).unwrap();
    assert_eq!(log.lines().count(), 2);
    for line in log.lines() { assert!(line.contains(&format!("\"verb\":\"rereview\",\"output_tokens\":{},\"baseline_tokens\":{}", (body.chars().count()+2)/4, (b+2)/4)), "{line}"); }
}
