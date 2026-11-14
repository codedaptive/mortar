//! save_report: a written save names its commit and change ID, an unchanged one HEAD; refused files are listed
//! with the cap; a formatter that did not end Formatted adds how it ended; every report is exit 0, next join.

use std::path::PathBuf;
use std::time::Duration;

use gitio::Oid;
use oplog::ChangeId;
use snapshot::{FormatterResult, FormatterRun, RefusedFile, SaveOutcome};

use super::save_report;
use crate::report::{ExitCode, OutValue, Report};

const COMMIT: &str = "0123456789abcdef0123456789abcdef01234567";
const TREE: &str = "89abcdef0123456789abcdef0123456789abcdef";
const CHANGE: &str = "kkkkllllmmmmnnnnoooopppprrrrssss";

fn outcome(written: bool, refused: Vec<RefusedFile>) -> SaveOutcome {
    SaveOutcome {
        commit: Oid::parse(COMMIT).unwrap(),
        tree: Oid::parse(TREE).unwrap(),
        written,
        change_id: written.then(|| ChangeId::parse(CHANGE).unwrap()),
        refused,
    }
}

fn run(result: FormatterResult) -> FormatterRun {
    FormatterRun {
        program: PathBuf::from("rustfmt"),
        args: Vec::new(),
        paths: vec![PathBuf::from("src/lib.rs")],
        result,
    }
}

fn formatter_lines(result: FormatterResult) -> Vec<String> {
    save_report(&outcome(true, Vec::new()), Some(&run(result))).lines[1..].to_vec()
}

#[test]
fn a_written_save_names_commit_and_change_id() {
    assert_eq!(
        save_report(&outcome(true, Vec::new()), None),
        Report {
            verb: "save".to_string(),
            exit: ExitCode::Done,
            outcome: "saved".to_string(),
            lines: vec![
                "saved commit 0123456789abcdef0123456789abcdef01234567 on the request's branch, change ID \
                 kkkkllllmmmmnnnnoooopppprrrrssss"
                    .to_string()
            ],
            owner: None,
            next: "mortar join".to_string(),
            fields: vec![
                ("commit".to_string(), OutValue::Text(COMMIT.to_string())),
                ("written".to_string(), OutValue::Bool(true)),
                ("change_id".to_string(), OutValue::Text(CHANGE.to_string())),
                ("refused".to_string(), OutValue::Texts(Vec::new())),
                ("cap".to_string(), OutValue::Null),
            ],
        }
    );
}

#[test]
fn a_written_save_without_a_change_id_says_none() {
    let mut saved = outcome(true, Vec::new());
    saved.change_id = None;
    let report = save_report(&saved, None);
    assert_eq!(
        report.lines,
        vec!["saved commit 0123456789abcdef0123456789abcdef01234567 on the request's branch, change ID none"
            .to_string()]
    );
    assert_eq!(report.fields[2], ("change_id".to_string(), OutValue::Null));
}

#[test]
fn an_unchanged_save_names_head() {
    assert_eq!(
        save_report(&outcome(false, Vec::new()), None),
        Report {
            verb: "save".to_string(),
            exit: ExitCode::Done,
            outcome: "unchanged".to_string(),
            lines: vec![
                "nothing changed since the last save; no commit written (HEAD is \
                 0123456789abcdef0123456789abcdef01234567)"
                    .to_string()
            ],
            owner: None,
            next: "mortar join".to_string(),
            fields: vec![
                ("commit".to_string(), OutValue::Text(COMMIT.to_string())),
                ("written".to_string(), OutValue::Bool(false)),
                ("change_id".to_string(), OutValue::Null),
                ("refused".to_string(), OutValue::Texts(Vec::new())),
                ("cap".to_string(), OutValue::Null),
            ],
        }
    );
}

#[test]
fn refused_files_are_listed_with_the_cap() {
    let refused = vec![
        RefusedFile { path: PathBuf::from("data/big.bin"), size: 2_000_000, cap: 1_048_576 },
        RefusedFile { path: PathBuf::from("logs/run.log"), size: 3_500_000, cap: 1_048_576 },
    ];
    let report = save_report(&outcome(false, refused), None);
    assert_eq!(report.exit, ExitCode::Done);
    assert_eq!(
        report.lines[1..].to_vec(),
        vec![
            "not saved: data/big.bin is 2000000 bytes, over the size cap of 1048576 bytes".to_string(),
            "not saved: logs/run.log is 3500000 bytes, over the size cap of 1048576 bytes".to_string(),
        ]
    );
    assert_eq!(
        report.fields[3..].to_vec(),
        vec![
            (
                "refused".to_string(),
                OutValue::Texts(vec!["data/big.bin".to_string(), "logs/run.log".to_string()])
            ),
            ("cap".to_string(), OutValue::Number(1_048_576)),
        ]
    );
}

#[test]
fn a_formatted_run_adds_no_line() {
    assert_eq!(formatter_lines(FormatterResult::Formatted), Vec::<String>::new());
}

#[test]
fn a_failed_formatter_names_its_exit_and_stderr() {
    assert_eq!(
        formatter_lines(FormatterResult::Failed { code: Some(2), stderr: "  error: bad token\n".to_string() }),
        vec![
            "the formatter rustfmt failed (exit 2); the files were saved as they are".to_string(),
            "formatter stderr: error: bad token".to_string(),
        ]
    );
}

#[test]
fn a_formatter_killed_by_a_signal_with_no_stderr_is_one_line() {
    assert_eq!(
        formatter_lines(FormatterResult::Failed { code: None, stderr: " \n".to_string() }),
        vec!["the formatter rustfmt failed (killed by a signal); the files were saved as they are".to_string()]
    );
}

#[test]
fn a_formatter_that_could_not_run_says_why() {
    assert_eq!(
        formatter_lines(FormatterResult::SpawnFailed { error: "No such file or directory (os error 2)".to_string() }),
        vec![
            "the formatter rustfmt could not run: No such file or directory (os error 2); the files were saved as \
             they are"
                .to_string()
        ]
    );
}

#[test]
fn a_timed_out_formatter_names_the_limit() {
    assert_eq!(
        formatter_lines(FormatterResult::TimedOut { limit: Duration::from_secs(30), stderr: "slow".to_string() }),
        vec![
            "the formatter rustfmt ran past its timeout of 30 s and was stopped; the files were saved as they are"
                .to_string()
        ]
    );
}
