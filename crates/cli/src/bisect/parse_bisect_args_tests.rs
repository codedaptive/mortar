//! parse_bisect_args: the two commits and --timeout before `--`, the test command after it verbatim, and each word
//! it refuses, the first failure in the contract's order winning.

use std::ffi::OsString;
use std::path::PathBuf;
use std::time::Duration;

use gitio::Oid;
use land::TestCommand;

use super::parse_bisect_args;
use crate::bisect::BisectArgs;

const LAST: &str = "4b825dc642cb6eb9a060e54bf8d69288fbee4904";
const FAILED: &str = "e69de29bb2d1d6434b8b29ae775ad8c2e48c5391";
const THIRD: &str = "9c1f3e2a7b6d5c4e3f2a1b0c9d8e7f6a5b4c3d2e";
const SHA256: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

fn parse(rest: &[&str], dash_dash: Option<usize>) -> Result<BisectArgs, String> {
    parse_bisect_args(&rest.iter().map(ToString::to_string).collect::<Vec<_>>(), dash_dash)
}

fn refused(rest: &[&str], dash_dash: Option<usize>) -> String {
    match parse(rest, dash_dash) {
        Err(reason) => reason,
        Ok(args) => panic!("{rest:?} at {dash_dash:?} gave {args:?}"),
    }
}

fn os(words: &[&str]) -> Vec<OsString> {
    words.iter().map(|word| OsString::from(*word)).collect()
}

#[test]
fn no_double_dash_is_refused_first() {
    assert_eq!(
        refused(&[LAST, FAILED, "--timeout", "5", "make"], None),
        "bisect needs `--` before the test command"
    );
    assert_eq!(refused(&[], None), "bisect needs `--` before the test command");
}

#[test]
fn both_commits_the_timeout_and_the_command_are_read() {
    assert_eq!(
        parse(&[LAST, FAILED, "--timeout", "30", "./check.sh", "fast"], Some(4)),
        Ok(BisectArgs {
            last_pass: Oid::parse(LAST).unwrap(),
            failed: Oid::parse(FAILED).unwrap(),
            timeout: Duration::from_secs(30),
            command: TestCommand { program: PathBuf::from("./check.sh"), args: os(&["fast"]) },
        })
    );
}

#[test]
fn the_timeout_may_come_before_the_commits_and_full_sha256_ids_are_commits() {
    let args = parse(&["--timeout", "600", SHA256, FAILED, "make"], Some(4)).unwrap();
    assert_eq!(args.last_pass, Oid::parse(SHA256).unwrap());
    assert_eq!(args.failed, Oid::parse(FAILED).unwrap());
    assert_eq!(args.timeout, Duration::from_secs(600));
    assert_eq!(args.command, TestCommand { program: PathBuf::from("make"), args: Vec::new() });
}

#[test]
fn words_after_the_double_dash_are_the_command_verbatim_global_flags_and_a_second_double_dash_included() {
    let rest = [LAST, FAILED, "--timeout", "5", "cargo", "test", "--actor", "x", "--json", "--key", "k", "--", "-q"];
    let args = parse(&rest, Some(4)).unwrap();
    assert_eq!(args.command.program, PathBuf::from("cargo"));
    assert_eq!(args.command.args, os(&["test", "--actor", "x", "--json", "--key", "k", "--", "-q"]));
}

#[test]
fn timeout_with_no_value_before_the_double_dash_is_refused() {
    assert_eq!(refused(&[LAST, FAILED, "--timeout", "make"], Some(3)), "--timeout needs a value");
}

#[test]
fn timeout_given_twice_is_refused() {
    assert_eq!(
        refused(&[LAST, FAILED, "--timeout", "5", "--timeout", "6", "make"], Some(6)),
        "--timeout is given twice"
    );
}

#[test]
fn a_dashed_word_before_the_double_dash_is_refused_by_name() {
    assert_eq!(
        refused(&[LAST, FAILED, "--timeout=5", "make"], Some(3)),
        "unexpected word \"--timeout=5\" before `--`"
    );
    assert_eq!(
        refused(&[LAST, "--verbose", FAILED, "--timeout", "5", "make"], Some(5)),
        "unexpected word \"--verbose\" before `--`"
    );
    assert_eq!(refused(&["-x", LAST, FAILED, "--timeout", "5", "make"], Some(5)), "unexpected word \"-x\" before `--`");
}

#[test]
fn a_dashed_word_wins_over_too_few_commits() {
    assert_eq!(refused(&[LAST, "-n", "make"], Some(2)), "unexpected word \"-n\" before `--`");
}

#[test]
fn fewer_than_two_commits_is_refused() {
    let reason = "bisect needs two commits: the last passing one and the failed one";
    assert_eq!(refused(&[LAST, "--timeout", "5", "make"], Some(3)), reason);
    assert_eq!(refused(&["--timeout", "5", "make"], Some(2)), reason);
    assert_eq!(refused(&[], Some(0)), reason);
}

#[test]
fn a_third_commit_word_is_refused_by_name() {
    assert_eq!(
        refused(&[LAST, FAILED, THIRD, "--timeout", "5", "make"], Some(5)),
        "unexpected word \"9c1f3e2a7b6d5c4e3f2a1b0c9d8e7f6a5b4c3d2e\" before `--`"
    );
    assert_eq!(refused(&["HEAD", FAILED, "x", "make"], Some(3)), "unexpected word \"x\" before `--`");
}

#[test]
fn a_word_that_is_not_a_full_commit_id_is_refused_last_pass_first() {
    assert_eq!(
        refused(&["HEAD", "abc1234", "--timeout", "5", "make"], Some(4)),
        "\"HEAD\" is not a full commit id (40 or 64 lowercase hex characters)"
    );
    assert_eq!(
        refused(&[LAST, "abc1234", "--timeout", "5", "make"], Some(4)),
        "\"abc1234\" is not a full commit id (40 or 64 lowercase hex characters)"
    );
    assert_eq!(
        refused(&["E69DE29BB2D1D6434B8B29AE775AD8C2E48C5391", FAILED, "--timeout", "5", "make"], Some(4)),
        "\"E69DE29BB2D1D6434B8B29AE775AD8C2E48C5391\" is not a full commit id (40 or 64 lowercase hex characters)"
    );
}

#[test]
fn a_bad_commit_wins_over_a_bad_timeout() {
    assert_eq!(
        refused(&["main", FAILED, "--timeout", "0", "make"], Some(4)),
        "\"main\" is not a full commit id (40 or 64 lowercase hex characters)"
    );
}

#[test]
fn no_timeout_is_refused() {
    assert_eq!(refused(&[LAST, FAILED, "make"], Some(2)), "bisect needs --timeout <seconds>");
}

#[test]
fn a_timeout_that_is_not_a_whole_number_of_seconds_above_zero_is_refused_by_value() {
    let cases = [
        ("0", "--timeout \"0\" is not a whole number of seconds above 0"),
        ("-5", "--timeout \"-5\" is not a whole number of seconds above 0"),
        ("five", "--timeout \"five\" is not a whole number of seconds above 0"),
        ("1.5", "--timeout \"1.5\" is not a whole number of seconds above 0"),
        ("", "--timeout \"\" is not a whole number of seconds above 0"),
    ];
    for (value, reason) in cases {
        assert_eq!(refused(&[LAST, FAILED, "--timeout", value, "make"], Some(4)), reason);
    }
}

#[test]
fn no_timeout_wins_over_an_empty_command() {
    assert_eq!(refused(&[LAST, FAILED], Some(2)), "bisect needs --timeout <seconds>");
}

#[test]
fn nothing_after_the_double_dash_is_refused() {
    assert_eq!(refused(&[LAST, FAILED, "--timeout", "5"], Some(4)), "bisect needs a test command after `--`");
}
