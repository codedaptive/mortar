//! snapshot_report: a snapshot that ran is exit 0 whether or not it wrote a commit; it names the commit and tree,
//! each file the size cap left out with its size and cap, and the formatter's run when one ran, its result in words
//! per FormatterResult arm.

use std::ffi::OsString;
use std::path::PathBuf;
use std::time::Duration;

use gitio::Oid;
use snapshot::{FormatterResult, FormatterRun, RefusedFile, SnapshotOutcome};

use super::snapshot_report;
use crate::report::{ExitCode, OutValue, Report};

const COMMIT: &str = "0123456789abcdef0123456789abcdef01234567";
const TREE: &str = "4b825dc642cb6eb9a060e54bf8d69288fbee4904";

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).expect("a valid oid")
}

fn outcome(written: bool, refused: Vec<RefusedFile>, formatted: Option<FormatterRun>) -> SnapshotOutcome {
    SnapshotOutcome { commit: oid(COMMIT), tree: oid(TREE), written, refused, formatted }
}

fn run(result: FormatterResult) -> FormatterRun {
    FormatterRun {
        program: PathBuf::from("rustfmt"),
        args: vec![OsString::from("--edition"), OsString::from("2021")],
        paths: vec![PathBuf::from("src/a.rs"), PathBuf::from("src/b.rs")],
        result,
    }
}

fn formatter_line(result: FormatterResult) -> String {
    let report = snapshot_report(&outcome(true, Vec::new(), Some(run(result))));
    assert_eq!(report.lines.len(), 2, "{report:?}");
    report.lines[1].clone()
}

#[test]
fn a_written_snapshot_is_done_naming_commit_and_tree() {
    assert_eq!(
        snapshot_report(&outcome(true, Vec::new(), None)),
        Report {
            verb: "snapshot".to_string(),
            exit: ExitCode::Done,
            outcome: "snapshotted".to_string(),
            lines: vec![
                "snapshotted commit 0123456789abcdef0123456789abcdef01234567 (tree \
                 4b825dc642cb6eb9a060e54bf8d69288fbee4904)"
                    .to_string()
            ],
            owner: None,
            next: "nothing to do".to_string(),
            fields: vec![
                ("commit".to_string(), OutValue::Text(COMMIT.to_string())),
                ("tree".to_string(), OutValue::Text(TREE.to_string())),
                ("written".to_string(), OutValue::Bool(true)),
                ("refused".to_string(), OutValue::Texts(Vec::new())),
            ],
        }
    );
}

#[test]
fn an_unchanged_tree_is_done_too() {
    let report = snapshot_report(&outcome(false, Vec::new(), None));
    assert_eq!(report.exit as i32, 0);
    assert_eq!(report.outcome, "unchanged");
    assert_eq!(
        report.lines,
        vec!["unchanged: the snapshot is commit 0123456789abcdef0123456789abcdef01234567 (tree \
              4b825dc642cb6eb9a060e54bf8d69288fbee4904)"
            .to_string()]
    );
    assert_eq!(report.next, "nothing to do");
    assert_eq!(report.fields[2], ("written".to_string(), OutValue::Bool(false)));
}

#[test]
fn each_refused_file_is_named_with_its_size_and_cap() {
    let refused = vec![
        RefusedFile { path: PathBuf::from("data/big.bin"), size: 2_000_000, cap: 1_048_576 },
        RefusedFile { path: PathBuf::from("logs/run.log"), size: 1_500_000, cap: 1_048_576 },
    ];
    let report = snapshot_report(&outcome(true, refused, None));
    assert_eq!(
        report.lines[1..],
        [
            "not snapshotted: data/big.bin is 2000000 bytes, over the size cap of 1048576 bytes".to_string(),
            "not snapshotted: logs/run.log is 1500000 bytes, over the size cap of 1048576 bytes".to_string(),
        ]
    );
    assert_eq!(
        report.fields[3],
        ("refused".to_string(), OutValue::Texts(vec!["data/big.bin".to_string(), "logs/run.log".to_string()]))
    );
    assert_eq!(report.exit, ExitCode::Done);
}

#[test]
fn no_formatter_run_adds_no_line() {
    assert_eq!(snapshot_report(&outcome(true, Vec::new(), None)).lines.len(), 1);
}

#[test]
fn a_formatted_run_names_the_command_and_paths() {
    assert_eq!(
        formatter_line(FormatterResult::Formatted),
        "formatter rustfmt --edition 2021 on src/a.rs, src/b.rs: formatted"
    );
}

#[test]
fn a_failed_run_names_its_exit_and_that_the_files_are_kept_as_they_are() {
    assert_eq!(
        formatter_line(FormatterResult::Failed { code: Some(2), stderr: "error: bad".to_string() }),
        "formatter rustfmt --edition 2021 on src/a.rs, src/b.rs: failed (exit 2); the files were snapshotted as they \
         are"
    );
    assert_eq!(
        formatter_line(FormatterResult::Failed { code: None, stderr: String::new() }),
        "formatter rustfmt --edition 2021 on src/a.rs, src/b.rs: failed (killed by a signal); the files were \
         snapshotted as they are"
    );
}

#[test]
fn a_run_that_could_not_start_says_so() {
    assert_eq!(
        formatter_line(FormatterResult::SpawnFailed { error: "no such file".to_string() }),
        "formatter rustfmt --edition 2021 on src/a.rs, src/b.rs: could not run: no such file; the files were \
         snapshotted as they are"
    );
}

#[test]
fn a_run_past_its_timeout_names_the_limit() {
    assert_eq!(
        formatter_line(FormatterResult::TimedOut { limit: Duration::from_secs(30), stderr: String::new() }),
        "formatter rustfmt --edition 2021 on src/a.rs, src/b.rs: ran past its timeout of 30 s and was stopped; the \
         files were snapshotted as they are"
    );
}
