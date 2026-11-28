//! hook_refusal_report: the refusal line, the verb's nothing-moved line, the hook's output line by line, and the
//! hook, code, stdout and stderr fields.

use gitio::HookRefusal;

use super::hook_refusal_report;
use crate::report::{ExitCode, OutValue, Report};

struct Row {
    name: &'static str,
    verb: &'static str,
    refusal: HookRefusal,
    lines: &'static [&'static str],
    code: OutValue,
    stdout: &'static str,
    stderr: &'static str,
    owner: &'static str,
    next: &'static str,
}

#[test]
fn a_hook_refusal_reports_the_hook_its_code_and_its_output() {
    let rows = [
        Row {
            name: "pre-commit with a code and two-line output",
            verb: "save",
            refusal: HookRefusal {
                hook: "pre-commit",
                code: Some(7),
                stdout: b"out one\nout two\n".to_vec(),
                stderr: b"err one\nerr two\n".to_vec(),
            },
            lines: &[
                "the pre-commit hook refused the commit, exit 7",
                "nothing moved",
                "stdout: out one",
                "stdout: out two",
                "stderr: err one",
                "stderr: err two",
            ],
            code: OutValue::Number(7),
            stdout: "out one\nout two\n",
            stderr: "err one\nerr two\n",
            owner: "whoever maintains the pre-commit hook",
            next: "fix what the pre-commit hook reports, then mr save again",
        },
        Row {
            name: "commit-msg killed by a signal with empty stdout",
            verb: "squash",
            refusal: HookRefusal { hook: "commit-msg", code: None, stdout: Vec::new(), stderr: b"bad message\n".to_vec() },
            lines: &["the commit-msg hook refused the commit, killed by a signal", "nothing moved", "stderr: bad message"],
            code: OutValue::Null,
            stdout: "",
            stderr: "bad message\n",
            owner: "whoever maintains the commit-msg hook",
            next: "fix what the commit-msg hook reports, then mr squash again",
        },
        Row {
            name: "non-UTF-8 output",
            verb: "join",
            refusal: HookRefusal {
                hook: "pre-commit",
                code: Some(1),
                stdout: b"ok\xffend\n".to_vec(),
                stderr: b"\xfe".to_vec(),
            },
            lines: &[
                "the pre-commit hook refused the commit, exit 1",
                "nothing moved",
                "stdout: ok\u{FFFD}end",
                "stderr: \u{FFFD}",
            ],
            code: OutValue::Number(1),
            stdout: "ok\u{FFFD}end\n",
            stderr: "\u{FFFD}",
            owner: "whoever maintains the pre-commit hook",
            next: "fix what the pre-commit hook reports, then mr join again",
        },
    ];
    for row in rows {
        let hook = row.refusal.hook;
        assert_eq!(
            hook_refusal_report(row.verb, &row.refusal, "nothing moved"),
            Report {
                verb: row.verb.to_string(),
                exit: ExitCode::Refused,
                outcome: "refused".to_string(),
                lines: row.lines.iter().map(|line| line.to_string()).collect(),
                owner: Some(row.owner.to_string()),
                next: row.next.to_string(),
                fields: vec![
                    ("hook".to_string(), OutValue::Text(hook.to_string())),
                    ("code".to_string(), row.code),
                    ("stdout".to_string(), OutValue::Text(row.stdout.to_string())),
                    ("stderr".to_string(), OutValue::Text(row.stderr.to_string())),
                ],
            },
            "{}",
            row.name
        );
    }
}
