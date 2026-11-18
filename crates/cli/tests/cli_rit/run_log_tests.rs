//! run_log: with one op in the log, plain `mr log` lists it and prints no records, and `mr --json log` puts the log's
//! own line in "records", byte for byte (S-114): for a line with no values and no remote mark, and for a done push
//! line carrying values and a remote mark (S-103), so --json gives the values and the mark exactly as oplog wrote
//! them.
//! RIT (S-135, S-136): the mortar binary runs as a child in a throwaway repository (testkit's TestRepo, removed after
//! the test) whose config `mr init` writes first; real git answers its common dir, and oplog reads a real op log file
//! holding one line written here by hand. Children run with GIT_CONFIG_GLOBAL=/dev/null and GIT_CONFIG_NOSYSTEM=1.
//! Nagatha's from landing. log's usage cases are ZUTs in crates/cli/src/log/run_log_tests.rs.

use std::fs;
use std::path::Path;
use std::process::{Command, Output, Stdio};

use testkit::TestRepo;

/// The next step of a log that listed ops (log_report's "listed").
const LISTED_NEXT: &str = "mr restore <op id> <full ref name>... --actor <name> sets refs back as of one of these ops";

/// One op with no values and no remote mark, in oplog's key order.
const OP_1: &str = r#"{"op_id":"op-1","key":"k","kind":"save","actor":"core/fix-1","reason":"mortar save","at":5,"phase":"intent","moves":[{"ref":"refs/heads/req/core/fix-1","old":null,"new":"cccccccccccccccccccccccccccccccccccccccc"}]}"#;

/// One done push carrying a folder_tree value and the remote mark "origin", in oplog's key order
/// (encode_marked_record: the record, then "values", then "remote" last).
const MARKED_OP: &str = r#"{"op_id":"op-2","key":"publish-k","kind":"push","actor":"core/fix-1","reason":"mortar publish","at":6,"phase":"done","moves":[{"ref":"refs/heads/main","old":"cccccccccccccccccccccccccccccccccccccccc","new":"dddddddddddddddddddddddddddddddddddddddd"}],"result":{"outcome":"moved"},"values":[{"name":"folder_tree","value":"eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee"}],"remote":"origin"}"#;

fn mortar(cwd: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_mortar")).current_dir(cwd).env_clear()
        .env("PATH", std::env::var_os("PATH").unwrap()).env("HOME", cwd).env("TMPDIR", cwd)
        .env("GIT_CONFIG_NOSYSTEM", "1").env("GIT_CONFIG_GLOBAL", "/dev/null")
        .args(args).stdin(Stdio::null()).output().unwrap()
}

/// The binary's `mr log` (plain, or with --json) in a throwaway repository whose op log holds the one line `line`.
fn log_of_one_op(line: &str, json: bool) -> Output {
    let repo = TestRepo::new().expect("a throwaway repository");
    let init = mortar(&repo.path, &[
        "--actor", "harness", "init", "--origin", "main", "--integration", "integration", "--remote", "origin",
    ]);
    assert!(init.status.success(), "init: {init:?}");
    fs::write(repo.path.join(".git/mortar/oplog.jsonl"), format!("{line}\n")).expect("the op log");
    let output = mortar(&repo.path, if json { &["--json", "log"] } else { &["log"] });
    repo.close().expect("the repository is removed");
    output
}

/// Plain stdout is the report's lines then its next step; fields are --json only, so nothing else may be printed.
fn assert_plain(output: &Output, lines: &[&str]) {
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    let expected: String = lines.iter().map(|l| format!("{l}\n")).collect();
    assert_eq!(String::from_utf8_lossy(&output.stdout), format!("{expected}next: {LISTED_NEXT}\n"));
}

/// The one JSON line: exit 0, outcome "listed", and "records" last holding exactly `line`, byte for byte.
fn assert_json_records(output: &Output, line: &str) {
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.starts_with(r#"{"verb":"log","ok":true,"outcome":"listed","exit":0,"#), "{stdout}");
    assert!(stdout.ends_with(&format!(",\"records\":[{line}]}}\n")), "{stdout}");
}

#[test]
fn plain_log_lists_the_op_with_no_records() {
    assert_plain(&log_of_one_op(OP_1, false), &[
        "op-1 intent save by core/fix-1 at 5 key k: mortar save",
        "  refs/heads/req/core/fix-1 absent -> cccccccccccccccccccccccccccccccccccccccc",
    ]);
}

#[test]
fn json_log_holds_the_logs_own_line() {
    assert_json_records(&log_of_one_op(OP_1, true), OP_1);
}

#[test]
fn plain_log_lists_a_marked_op_with_no_records() {
    assert_plain(&log_of_one_op(MARKED_OP, false), &[
        "op-2 done push by core/fix-1 at 6 key publish-k: mortar publish",
        "  refs/heads/main cccccccccccccccccccccccccccccccccccccccc -> dddddddddddddddddddddddddddddddddddddddd",
    ]);
}

#[test]
fn json_log_holds_a_marked_lines_values_and_remote() {
    assert_json_records(&log_of_one_op(MARKED_OP, true), MARKED_OP);
}
