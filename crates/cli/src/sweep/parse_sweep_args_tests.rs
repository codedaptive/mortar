//! parse_sweep_args: --idle-days at most once with a whole number (0 allowed), --ref repeated with a value each, kept
//! in order and duplicates kept as given; anything else and any `--` refused.

use super::parse_sweep_args;
use crate::sweep::SweepArgs;

fn parse(rest: &[&str], dash_dash: Option<usize>) -> Result<SweepArgs, String> {
    parse_sweep_args(&rest.iter().map(ToString::to_string).collect::<Vec<_>>(), dash_dash)
}

fn refused(rest: &[&str]) -> String {
    match parse(rest, None) {
        Err(reason) => reason,
        Ok(args) => panic!("{rest:?} gave {args:?}"),
    }
}

fn args(idle_days: Option<u64>, refs: &[&str]) -> SweepArgs {
    SweepArgs { idle_days, refs: refs.iter().map(ToString::to_string).collect() }
}

#[test]
fn nothing_sweeps_refs_only() {
    assert_eq!(parse(&[], None), Ok(args(None, &[])));
}

#[test]
fn zero_idle_days_is_allowed() {
    assert_eq!(parse(&["--idle-days", "0"], None), Ok(args(Some(0), &[])));
}

#[test]
fn idle_days_with_two_refs_keeps_them_in_order() {
    assert_eq!(
        parse(&["--idle-days", "7", "--ref", "refs/t3/b", "--ref", "refs/t3/a"], None),
        Ok(args(Some(7), &["refs/t3/b", "refs/t3/a"]))
    );
    assert_eq!(
        parse(&["--ref", "refs/t3/b", "--idle-days", "7", "--ref", "refs/t3/a"], None),
        Ok(args(Some(7), &["refs/t3/b", "refs/t3/a"]))
    );
}

#[test]
fn a_ref_given_twice_is_kept_twice() {
    assert_eq!(
        parse(&["--ref", "refs/t3/a", "--ref", "refs/t3/a"], None),
        Ok(args(None, &["refs/t3/a", "refs/t3/a"]))
    );
}

#[test]
fn idle_days_twice_is_refused() {
    assert_eq!(refused(&["--idle-days", "1", "--idle-days", "2"]), "--idle-days is given twice");
}

#[test]
fn idle_days_without_a_value_is_refused() {
    assert_eq!(refused(&["--idle-days"]), "--idle-days needs a whole number of days");
}

#[test]
fn idle_days_that_is_not_a_whole_number_is_refused_naming_it() {
    assert_eq!(refused(&["--idle-days", "seven"]), "--idle-days needs a whole number of days, not \"seven\"");
    assert_eq!(refused(&["--idle-days", "1.5"]), "--idle-days needs a whole number of days, not \"1.5\"");
    assert_eq!(refused(&["--idle-days", "-1"]), "--idle-days needs a whole number of days, not \"-1\"");
}

#[test]
fn idle_days_with_a_plus_sign_is_refused() {
    assert_eq!(refused(&["--idle-days", "+1"]), "--idle-days needs a whole number of days, not \"+1\"");
}

#[test]
fn ref_without_a_value_is_refused() {
    assert_eq!(refused(&["--ref"]), "--ref needs a full ref name");
    assert_eq!(refused(&["--idle-days", "3", "--ref"]), "--ref needs a full ref name");
}

#[test]
fn a_stray_word_is_refused_by_name() {
    assert_eq!(refused(&["now"]), "unexpected word \"now\"");
    assert_eq!(refused(&["--idle-days", "3", "--all"]), "unexpected word \"--all\"");
}

#[test]
fn a_double_dash_is_refused() {
    assert_eq!(parse(&[], Some(0)), Err("sweep takes no `--`".to_string()));
    assert_eq!(parse(&["--idle-days", "3"], Some(2)), Err("sweep takes no `--`".to_string()));
}
