//! with_path_warning: a warning becomes the report's last line, so plain output prints it before `next:`; no warning
//! leaves the report as it was.

use super::with_path_warning;
use crate::report::{render_plain, ExitCode, OutValue, Report};

fn installed() -> Report {
    Report {
        verb: "guard".to_string(),
        exit: ExitCode::Done,
        outcome: "installed".to_string(),
        lines: vec![
            "the guard is installed in /r/.git/hooks".to_string(),
            "it guards refs/heads/develop and refs/heads/release, and pushes of them to upstream".to_string(),
        ],
        owner: None,
        next: "nothing to do: git now refuses moves of develop and release that Mortar did not make".to_string(),
        fields: vec![("hooks_dir".to_string(), OutValue::Text("/r/.git/hooks".to_string()))],
    }
}

const WARNING: &str = "no mr is on PATH: run Mortar as mortar (/b/target/debug/mortar)";

#[test]
fn a_warning_is_appended_as_the_last_line_and_nothing_else_changes() {
    assert_eq!(
        with_path_warning(installed(), Some(WARNING.to_string())),
        Report {
            verb: "guard".to_string(),
            exit: ExitCode::Done,
            outcome: "installed".to_string(),
            lines: vec![
                "the guard is installed in /r/.git/hooks".to_string(),
                "it guards refs/heads/develop and refs/heads/release, and pushes of them to upstream".to_string(),
                "no mr is on PATH: run Mortar as mortar (/b/target/debug/mortar)".to_string(),
            ],
            owner: None,
            next: "nothing to do: git now refuses moves of develop and release that Mortar did not make".to_string(),
            fields: vec![("hooks_dir".to_string(), OutValue::Text("/r/.git/hooks".to_string()))],
        }
    );
}

#[test]
fn a_warning_prints_before_next_in_plain_output() {
    assert_eq!(
        render_plain(&with_path_warning(installed(), Some(WARNING.to_string()))),
        "the guard is installed in /r/.git/hooks\n\
         it guards refs/heads/develop and refs/heads/release, and pushes of them to upstream\n\
         no mr is on PATH: run Mortar as mortar (/b/target/debug/mortar)\n\
         next: nothing to do: git now refuses moves of develop and release that Mortar did not make\n"
    );
}

#[test]
fn no_warning_leaves_the_report_unchanged() {
    assert_eq!(with_path_warning(installed(), None), installed());
}
