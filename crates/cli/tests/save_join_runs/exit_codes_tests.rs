//! `mr join` answers refused (3), conflict (1), busy (4) and usage (2), plain and JSON, the plain text ending in a
//! "next: " line; the program help, by `--help` and `help`, lists the exit codes before the tokens saved lines.
use crate::{json, support::{raw, Fixture, ZONE}};
use oplog::{KillPidProbe, LockName, LockRecord};
use std::process::Output;

#[derive(Clone, Copy)]
enum Case { Refused, Conflict, Busy, Usage }

fn run_case(case: Case, json: bool) -> Output {
    let f = Fixture::new();
    let args: &[&str] = match (case, json) {
        (Case::Usage, false) => &["join", "extra-arg"],
        (Case::Usage, true) => &["--json", "join", "extra-arg"],
        (_, false) => &["join"],
        (_, true) => &["--json", "join"],
    };
    match case {
        Case::Refused => { raw(&f.git, &["update-ref", "-d", ZONE]); }
        Case::Conflict => { f.save_topic(); f.advance_zone(true); }
        Case::Usage => {}
        Case::Busy => {
            let holder = LockRecord { pid: std::process::id(), host: oplog::host_name().unwrap(), key: "exit-codes-holder".into(), since: 123 };
            let held = oplog::acquire_lock(&f.common, &LockName::Join("core".into()), &holder, &KillPidProbe).unwrap();
            let output = f.app(args);
            oplog::release_lock(&held).unwrap();
            return output;
        }
    }
    f.app(args)
}

fn check_case(case: Case, json_format: bool, exit: i32, outcome: &str) {
    let output = run_case(case, json_format);
    if json_format {
        json::check(output, "join", exit, outcome);
    } else {
        let text = std::str::from_utf8(&output.stdout).unwrap();
        assert!(text.lines().last().is_some_and(|line| line.starts_with("next: ")), "{text}");
        assert_eq!(output.status.code(), Some(exit), "{text}");
    }
}

macro_rules! exit_case {
    ($name:ident, $case:ident, $json:literal, $exit:literal, $outcome:literal) => {
        #[test]
        fn $name() { check_case(Case::$case, $json, $exit, $outcome); }
    };
}

exit_case!(refused_plain, Refused, false, 3, "refused");
exit_case!(refused_json, Refused, true, 3, "refused");
exit_case!(conflict_plain, Conflict, false, 1, "conflicted");
exit_case!(conflict_json, Conflict, true, 1, "conflicted");
exit_case!(busy_plain, Busy, false, 4, "busy");
exit_case!(busy_json, Busy, true, 4, "busy");
exit_case!(usage_plain, Usage, false, 2, "usage");
exit_case!(usage_json, Usage, true, 2, "usage");

fn check_help(args: &[&str], json_format: bool) {
    let f = Fixture::new();
    let output = f.app(args);
    let lines: Vec<String> = if json_format {
        json::check(output, "help", 0, "help").strings("lines").to_vec()
    } else {
        assert_eq!(output.status.code(), Some(0));
        let text = std::str::from_utf8(&output.stdout).unwrap();
        assert!(text.lines().last().is_some_and(|line| line.starts_with("next: ")), "{text}");
        text.lines().map(str::to_string).collect()
    };
    let heading = lines.iter().position(|line| line == "Exit codes").expect("Exit codes heading");
    assert_eq!(lines[heading - 1], "");
    let savings = lines.iter().position(|line| line.starts_with("Tokens saved:")).unwrap();
    assert!(heading < savings);
    for token in ["  3  refused", "  4  busy", "  5  failed"] {
        assert!(lines[heading + 1..savings].iter().any(|line| line.starts_with(token)), "missing {token}");
    }
}

macro_rules! help_case {
    ($name:ident, $json:literal, $($arg:literal),+) => {
        #[test]
        fn $name() { check_help(&[$($arg),+], $json); }
    };
}

help_case!(help_flag_plain, false, "--help");
help_case!(help_flag_json, true, "--json", "--help");
help_case!(help_verb_plain, false, "help");
help_case!(help_verb_json, true, "--json", "help");
