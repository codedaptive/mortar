//! Tests of plan_placement: the skip, each thing that can stand at `~/.local/bin/mortar`, and the refusal (spec 07
//! item 1).

use std::path::PathBuf;

use super::{plan_placement, BinaryAt, InstallFacts, InstallItem, InstallPaths, InstallStep, LinkAt};
use crate::report::ExitCode;

fn paths() -> InstallPaths {
    InstallPaths {
        bin_dir: PathBuf::from("/h/.local/bin"),
        mortar: PathBuf::from("/h/.local/bin/mortar"),
        mr: PathBuf::from("/h/.local/bin/mr"),
        state_dir: PathBuf::from("/h/.local/state/mortar"),
        record: PathBuf::from("/h/.local/state/mortar/install.json"),
    }
}

fn facts(binary_at: BinaryAt) -> InstallFacts {
    InstallFacts {
        this: Some(PathBuf::from("/src/mortar")),
        this_hash: Some("0123456789abcdef".to_string()),
        binary_at,
        link_at: LinkAt::Absent,
        first_mr: None,
        first_mortar: None,
        first_mr_at: None,
        bin_at: Some(0),
        exe_dir: Some(PathBuf::from("/src")),
        detected: Vec::new(),
        skills: Vec::new(),
        record: Vec::new(),
    }
}

fn item(action: &str, note: &str) -> InstallItem {
    InstallItem { action: action.to_string(), path: PathBuf::from("/h/.local/bin/mortar"), note: note.to_string() }
}

fn place() -> Option<InstallStep> {
    Some(InstallStep::PlaceBinary { from: PathBuf::from("/src/mortar"), to: PathBuf::from("/h/.local/bin/mortar") })
}

#[test]
fn a_skip_reason_skips_whatever_stands_there() {
    for at in [BinaryAt::Absent, BinaryAt::Same, BinaryAt::Mortar, BinaryAt::Foreign] {
        let planned = plan_placement(&facts(at), &paths(), Some("--no-place")).unwrap();
        assert_eq!(planned, (None, item("skipped", "--no-place")));
    }
}

#[test]
fn nothing_there_is_placed() {
    assert_eq!(plan_placement(&facts(BinaryAt::Absent), &paths(), None).unwrap(), (place(), item("placed", "")));
}

#[test]
fn another_mortar_build_is_replaced() {
    assert_eq!(plan_placement(&facts(BinaryAt::Mortar), &paths(), None).unwrap(), (place(), item("replaced", "")));
}

#[test]
fn this_binary_is_unchanged_with_no_step() {
    assert_eq!(plan_placement(&facts(BinaryAt::Same), &paths(), None).unwrap(), (None, item("unchanged", "")));
}

#[test]
fn a_foreign_program_is_refused() {
    let report = plan_placement(&facts(BinaryAt::Foreign), &paths(), None).unwrap_err();
    assert_eq!(report.verb, "install");
    assert_eq!(report.exit, ExitCode::Refused);
    assert_eq!(report.outcome, "refused");
    assert_eq!(report.lines, ["/h/.local/bin/mortar is another program, not Mortar", "nothing changed"]);
    assert_eq!(report.owner.as_deref(), Some("whoever put /h/.local/bin/mortar there"));
    assert_eq!(report.next, "move /h/.local/bin/mortar aside, or install with --no-place");
}
