//! Tests of plan_skills: each thing that can stand at a skill path, the record's say, the wiring note and the
//! refusal. The text is "foobar", whose FNV-1a 64 is the published 85944171f73967e8.

use std::path::PathBuf;

use super::{
    plan_skills, BinaryAt, InstallFacts, InstallItem, InstallStep, LinkAt, RecordItem, SkillAt, HARNESSES,
};
use crate::report::ExitCode;

const CLAUDE: &str = "/h/.claude/skills/mortar/SKILL.md";
const CURSOR: &str = "/h/.cursor/skills/mortar/SKILL.md";

fn facts(claude: SkillAt, cursor: SkillAt) -> InstallFacts {
    InstallFacts {
        this: None,
        this_hash: None,
        binary_at: BinaryAt::Absent,
        link_at: LinkAt::Absent,
        first_mr: None,
        first_mortar: None,
        first_mr_at: None,
        bin_at: None,
        exe_dir: None,
        detected: vec![&HARNESSES[0], &HARNESSES[2]],
        skills: vec![(&HARNESSES[0], PathBuf::from(CLAUDE), claude), (&HARNESSES[2], PathBuf::from(CURSOR), cursor)],
        record: Vec::new(),
    }
}

fn item(action: &str, path: &str, note: &str) -> InstallItem {
    InstallItem { action: action.to_string(), path: PathBuf::from(path), note: note.to_string() }
}

fn write(path: &str) -> InstallStep {
    InstallStep::WriteSkill { path: PathBuf::from(path), text: "foobar".to_string() }
}

fn other() -> SkillAt {
    SkillAt::File { hash: "0000000000000001".to_string() }
}

#[test]
fn absent_files_are_written_for_each_chosen_harness() {
    let chosen = [&HARNESSES[0], &HARNESSES[2]];
    let (steps, items) = plan_skills(&facts(SkillAt::Absent, SkillAt::Absent), &chosen, "foobar", None).unwrap();
    assert_eq!(steps, [write(CLAUDE), write(CURSOR)]);
    assert_eq!(items, [item("written", CLAUDE, "claude"), item("written", CURSOR, "cursor")]);
}

#[test]
fn only_chosen_harnesses_are_planned() {
    let chosen = [&HARNESSES[2]];
    let (steps, items) = plan_skills(&facts(SkillAt::Absent, SkillAt::Absent), &chosen, "foobar", None).unwrap();
    assert_eq!(steps, [write(CURSOR)]);
    assert_eq!(items, [item("written", CURSOR, "cursor")]);
}

#[test]
fn the_same_text_is_unchanged_with_no_step() {
    let same = SkillAt::File { hash: "85944171f73967e8".to_string() };
    let (steps, items) = plan_skills(&facts(same, SkillAt::Absent), &[&HARNESSES[0]], "foobar", None).unwrap();
    assert!(steps.is_empty());
    assert_eq!(items, [item("unchanged", CLAUDE, "claude")]);
}

#[test]
fn a_file_the_record_lists_is_replaced() {
    let mut facts = facts(other(), SkillAt::Absent);
    facts.record = vec![RecordItem::File { path: PathBuf::from(CLAUDE), hash: "0000000000000002".to_string() }];
    let (steps, items) = plan_skills(&facts, &[&HARNESSES[0]], "foobar", None).unwrap();
    assert_eq!(steps, [write(CLAUDE)]);
    assert_eq!(items, [item("replaced", CLAUDE, "claude")]);
}

#[test]
fn a_file_the_record_does_not_list_is_refused() {
    let mut facts = facts(SkillAt::Absent, other());
    facts.record = vec![RecordItem::Dir { path: PathBuf::from(CURSOR) }];
    let report = plan_skills(&facts, &[&HARNESSES[0], &HARNESSES[2]], "foobar", None).unwrap_err();
    assert_eq!(report.exit, ExitCode::Refused);
    assert_eq!(report.outcome, "refused");
    assert_eq!(report.lines, [format!("{CURSOR} is not Mortar's skill file"), "nothing changed".to_string()]);
    assert_eq!(report.owner, Some(format!("whoever wrote {CURSOR}")));
    assert_eq!(report.next, format!("move {CURSOR} aside, or leave cursor out of --target"));
}

#[test]
fn something_not_a_file_is_refused_even_when_recorded() {
    let mut facts = facts(SkillAt::NotFile, SkillAt::Absent);
    facts.record = vec![RecordItem::File { path: PathBuf::from(CLAUDE), hash: "85944171f73967e8".to_string() }];
    assert_eq!(plan_skills(&facts, &[&HARNESSES[0]], "foobar", None).unwrap_err().exit, ExitCode::Refused);
}

#[test]
fn the_wiring_note_comes_last_with_no_path() {
    let note = Some("none picked".to_string());
    let (steps, items) = plan_skills(&facts(SkillAt::Absent, SkillAt::Absent), &[], "foobar", note).unwrap();
    assert!(steps.is_empty());
    assert_eq!(items, [item("wiring skipped", "", "none picked")]);
}
