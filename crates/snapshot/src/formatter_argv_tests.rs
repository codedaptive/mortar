use crate::formatter::Formatter;
use crate::formatter_argv::formatter_argv;
use std::ffi::OsString;
use std::path::PathBuf;

fn formatter(args: &[&str]) -> Formatter {
    Formatter { program: PathBuf::from("rustfmt"), args: args.iter().map(OsString::from).collect() }
}

fn paths(list: &[&str]) -> Vec<PathBuf> {
    list.iter().map(PathBuf::from).collect()
}

fn argv(list: &[&str]) -> Option<Vec<OsString>> {
    Some(list.iter().map(OsString::from).collect())
}

#[test]
fn args_then_paths_in_order_without_the_program() {
    let formatter = formatter(&["--edition", "2021", "--"]);
    assert_eq!(
        formatter_argv(Some(&formatter), &paths(&["src/a.rs", "b c.rs"])),
        argv(&["--edition", "2021", "--", "src/a.rs", "b c.rs"])
    );
}

#[test]
fn no_args_gives_only_the_paths() {
    assert_eq!(formatter_argv(Some(&formatter(&[])), &paths(&["x.rs"])), argv(&["x.rs"]));
}

#[test]
fn no_formatter_runs_nothing() {
    assert_eq!(formatter_argv(None, &paths(&["x.rs"])), None);
}

#[test]
fn no_paths_runs_nothing() {
    assert_eq!(formatter_argv(Some(&formatter(&["--"])), &[]), None);
}
