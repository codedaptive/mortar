//! run_guard_hook: hook input that is not UTF-8 is refused (exit 1) by run_guard_hook's read of its input, before
//! land::guard_decide, with no ref moved. With MORTAR_OP set and the op log unreadable (its path a directory), a
//! reference-transaction committed or aborted is let through (exit 0, nothing printed) without reading the op log,
//! and a prepared one, guarded line or not, fails on the op log (exit 5), with no ref moved.
//! ZIT (S-119: the subject decides the group): each test runs the mortar binary with the hook's words and input, as
//! git would, in a throwaway repository.
//! The installed-hook and marked-op cases stay RITs in tests/operational_rit/guard.rs.
#[allow(dead_code)]
#[path = "cli_rit/json.rs"]
mod json;
#[allow(dead_code)]
#[path = "cli_rit/support.rs"]
mod support;
#[allow(dead_code)]
#[path = "operational_rit/cid.rs"]
mod cid;
#[allow(dead_code)]
#[path = "operational_rit/refs.rs"]
mod refs;
#[allow(dead_code)]
#[path = "operational_rit/guard_fixture.rs"]
mod guard_fixture;

use std::fs;

use guard_fixture::Guard;
use json::check;

#[test]
fn guard_invalid_utf8_input_refuses_without_moving_any_ref() {
    let g = Guard::new(); let before = g.branches();
    let report = check(g.hook(&["--json", "guard-hook", "reference-transaction", "prepared"], b"\xff"),
        "guard-hook", 1, "refused");
    assert!(report.lines().contains("the guard cannot read the hook's input"));
    assert!(report.lines().contains("it is not UTF-8")); assert_eq!(report.text("owner"), "the harness that installs the guard");
    assert_eq!(g.branches(), before);
}

/// A Guard whose MORTAR_OP names an op and whose op log path is a directory, so any read of the op log fails.
fn unreadable_log() -> Guard {
    let mut g = Guard::new();
    g.f.git.env.push(("MORTAR_OP".into(), "op-guard-zit".into()));
    let log = oplog::oplog_path(&g.f.common);
    if log.exists() { fs::remove_file(&log).unwrap(); }
    fs::create_dir_all(&log).unwrap();
    g
}

fn line(g: &Guard, refname: &str) -> Vec<u8> { format!("{} {} {refname}\n", g.f.base, g.new).into_bytes() }

fn let_through_unread(state: &str) {
    let g = unreadable_log(); let before = g.branches();
    let output = g.hook(&["--json", "guard-hook", "reference-transaction", state], &line(&g, "refs/heads/integration"));
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert!(output.stdout.is_empty(), "{output:?}"); assert!(output.stderr.is_empty(), "{output:?}");
    assert_eq!(g.branches(), before);
}

fn prepared_reads_the_log(refname: &str) {
    let g = unreadable_log(); let before = g.branches();
    let report = check(g.hook(&["--json", "guard-hook", "reference-transaction", "prepared"], &line(&g, refname)),
        "guard-hook", 5, "failed");
    assert!(report.lines().contains("mortar/oplog.jsonl"), "{}", report.lines());
    assert_eq!(g.branches(), before);
}

#[test]
fn guard_committed_lets_through_without_reading_the_op_log() { let_through_unread("committed"); }

#[test]
fn guard_aborted_lets_through_without_reading_the_op_log() { let_through_unread("aborted"); }

#[test]
fn guard_prepared_guarded_line_fails_on_an_unreadable_op_log() { prepared_reads_the_log("refs/heads/integration"); }

#[test]
fn guard_prepared_unguarded_line_fails_on_an_unreadable_op_log() { prepared_reads_the_log("refs/heads/feature"); }
