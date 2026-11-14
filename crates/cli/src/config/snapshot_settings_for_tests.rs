//! snapshot_settings_for: the formatter, the build-dir excludes and the cap, from a Config literal.

use std::collections::BTreeMap;
use std::ffi::OsString;
use std::path::PathBuf;
use std::time::Duration;

use snapshot::Formatter;

use super::{snapshot_settings_for, Config, FormatterConfig};

fn config(build_dirs: &[&str], formatter: Option<FormatterConfig>) -> Config {
    Config {
        origin_branch: "main".to_string(),
        integration_branch: "integration".to_string(),
        remote: "origin".to_string(),
        zones: BTreeMap::new(),
        root_files: Vec::new(),
        build_dirs: build_dirs.iter().map(|d| d.to_string()).collect(),
        cap_bytes: 1,
        floor_bytes: 1,
        formatter,
    }
}

fn formatter_config(command: &[&str], timeout_seconds: u64) -> Option<FormatterConfig> {
    Some(FormatterConfig { command: command.iter().map(|c| c.to_string()).collect(), timeout_seconds })
}

#[test]
fn no_formatter_gives_none() {
    assert_eq!(snapshot_settings_for(&config(&[], None)).formatter, None);
}

#[test]
fn formatter_takes_program_args_and_timeout_from_the_command() {
    let settings = snapshot_settings_for(&config(&[], formatter_config(&["rustfmt", "--edition", "2021"], 30)));
    assert_eq!(
        settings.formatter,
        Some(Formatter {
            program: PathBuf::from("rustfmt"),
            args: vec![OsString::from("--edition"), OsString::from("2021")],
            timeout: Duration::from_secs(30),
        })
    );
}

#[test]
fn a_one_word_command_has_no_args() {
    let settings = snapshot_settings_for(&config(&[], formatter_config(&["x"], 5)));
    assert_eq!(
        settings.formatter,
        Some(Formatter { program: PathBuf::from("x"), args: Vec::new(), timeout: Duration::from_secs(5) })
    );
}

#[test]
fn an_empty_command_gives_no_formatter() {
    assert_eq!(snapshot_settings_for(&config(&[], formatter_config(&[], 30))).formatter, None);
}

#[test]
fn a_build_dir_gives_its_top_level_and_nested_pathspecs() {
    assert_eq!(snapshot_settings_for(&config(&["target"], None)).excludes, vec!["target/", "*/target/*"]);
}

#[test]
fn trailing_slashes_are_trimmed_and_repeats_skipped() {
    assert_eq!(
        snapshot_settings_for(&config(&["target/", "build", "target"], None)).excludes,
        vec!["target/", "*/target/*", "build/", "*/build/*"]
    );
}

#[test]
fn a_slash_alone_is_skipped() {
    assert_eq!(snapshot_settings_for(&config(&["/"], None)).excludes, Vec::<String>::new());
}

#[test]
fn an_empty_build_dir_is_skipped() {
    assert_eq!(snapshot_settings_for(&config(&[""], None)).excludes, Vec::<String>::new());
}

#[test]
fn the_cap_is_passed_through() {
    let mut with_cap = config(&[], None);
    with_cap.cap_bytes = 52_428_800;
    assert_eq!(snapshot_settings_for(&with_cap).cap_bytes, 52_428_800);
}
