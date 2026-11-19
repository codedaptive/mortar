//! Tests of plan_install: the order of steps and items, the refusal order (placement, link, skills), the command the
//! skill text names, the "not on PATH" item and the next step's precedence.

use std::path::PathBuf;

use super::{
    plan_install, BinaryAt, InstallFacts, InstallItem, InstallOptions, InstallPaths, InstallStep, LinkAt, SkillAt,
    SkillLocation, HARNESSES,
};

const SKILL: &str = "/h/.claude/skills/mortar/SKILL.md";

fn paths() -> InstallPaths {
    InstallPaths {
        bin_dir: PathBuf::from("/h/.local/bin"),
        mortar: PathBuf::from("/h/.local/bin/mortar"),
        mr: PathBuf::from("/h/.local/bin/mr"),
        state_dir: PathBuf::from("/h/.local/state/mortar"),
        record: PathBuf::from("/h/.local/state/mortar/install.json"),
    }
}

fn facts() -> InstallFacts {
    InstallFacts {
        this: Some(PathBuf::from("/src/mortar")),
        this_hash: Some("0123456789abcdef".to_string()),
        binary_at: BinaryAt::Absent,
        link_at: LinkAt::Absent,
        first_mr: None,
        first_mortar: None,
        first_mr_at: None,
        bin_at: Some(0),
        exe_dir: Some(PathBuf::from("/src")),
        detected: vec![&HARNESSES[0]],
        skills: vec![(&HARNESSES[0], PathBuf::from(SKILL), SkillAt::Absent)],
        record: Vec::new(),
    }
}

fn options(short_name: bool) -> InstallOptions {
    InstallOptions { no_place: false, short_name, targets: None, yes: true, location: SkillLocation::Global }
}

fn actions(items: &[InstallItem]) -> Vec<&str> {
    items.iter().map(|item| item.action.as_str()).collect()
}

fn skill_text_of(steps: &[InstallStep]) -> &str {
    steps
        .iter()
        .find_map(|step| match step {
            InstallStep::WriteSkill { text, .. } => Some(text.as_str()),
            _ => None,
        })
        .unwrap()
}

#[test]
fn a_fresh_install_places_links_and_writes_naming_mr() {
    let plan = plan_install(&facts(), &options(false), &paths(), None, &[&HARNESSES[0]], None).unwrap();
    assert_eq!(plan.steps.len(), 3);
    assert_eq!(
        plan.steps[..2],
        [
            InstallStep::PlaceBinary { from: PathBuf::from("/src/mortar"), to: PathBuf::from("/h/.local/bin/mortar") },
            InstallStep::Link { at: PathBuf::from("/h/.local/bin/mr"), target: PathBuf::from("/h/.local/bin/mortar") },
        ]
    );
    assert!(skill_text_of(&plan.steps).contains("`mr status`"));
    assert_eq!(actions(&plan.items), ["placed", "linked", "written"]);
    assert_eq!(plan.next, "mr init, in the repository Mortar is to keep");
}

#[test]
fn fresh_plan_exposes_mr_command() {
    let plan = plan_install(&facts(), &options(false), &paths(), None, &[&HARNESSES[0]], None).unwrap();
    assert_eq!(plan.command, "mr");
}

#[test]
fn foreign_mr_before_bin_exposes_mortar_command_when_link_is_skipped() {
    let mut facts = facts();
    facts.first_mr = Some((PathBuf::from("/usr/bin/mr"), false));
    facts.first_mr_at = Some(0);
    facts.bin_at = Some(1);
    let plan = plan_install(&facts, &options(false), &paths(), None, &[&HARNESSES[0]], None).unwrap();
    assert_eq!(actions(&plan.items), ["placed", "link skipped", "written"]);
    assert_eq!(plan.command, "mortar");
}

#[test]
fn placement_refuses_before_the_link_and_the_skills() {
    let mut facts = facts();
    facts.binary_at = BinaryAt::Foreign;
    facts.link_at = LinkAt::Foreign;
    facts.skills[0].2 = SkillAt::NotFile;
    let report = plan_install(&facts, &options(true), &paths(), None, &[&HARNESSES[0]], None).unwrap_err();
    assert_eq!(report.lines[0], "/h/.local/bin/mortar is another program, not Mortar");
}

#[test]
fn the_link_refuses_before_the_skills() {
    let mut facts = facts();
    facts.link_at = LinkAt::Foreign;
    facts.skills[0].2 = SkillAt::NotFile;
    let report = plan_install(&facts, &options(true), &paths(), None, &[&HARNESSES[0]], None).unwrap_err();
    assert_eq!(report.lines[0], "/h/.local/bin/mr is another program, not Mortar's link");
}

#[test]
fn the_skills_refuse_last() {
    let mut facts = facts();
    facts.skills[0].2 = SkillAt::NotFile;
    let report = plan_install(&facts, &options(false), &paths(), None, &[&HARNESSES[0]], None).unwrap_err();
    assert_eq!(report.lines[0], format!("{SKILL} is not Mortar's skill file"));
}

#[test]
fn local_bin_off_path_adds_the_item_and_the_next_step_and_names_mortar() {
    let mut facts = facts();
    facts.bin_at = None;
    let note = Some("none picked".to_string());
    let plan = plan_install(&facts, &options(false), &paths(), None, &[&HARNESSES[0]], note).unwrap();
    assert_eq!(actions(&plan.items), ["placed", "linked", "written", "wiring skipped", "not on PATH"]);
    let last = plan.items.last().unwrap();
    assert_eq!(last.path, PathBuf::from("/h/.local/bin"));
    assert_eq!(last.note, "mortar and mr cannot be found by name, and the guard hooks run mortar");
    assert_eq!(plan.next, "add /h/.local/bin to PATH");
    assert!(skill_text_of(&plan.steps).contains("`mortar status`"));
}

#[test]
fn a_skipped_placement_without_mortar_on_path_takes_the_link_next_step() {
    let mut facts = facts();
    facts.bin_at = None;
    let skip = Some("--no-place".to_string());
    let plan = plan_install(&facts, &options(false), &paths(), skip, &[&HARNESSES[0]], None).unwrap();
    let write = InstallStep::WriteSkill { path: PathBuf::from(SKILL), text: skill_text_of(&plan.steps).to_string() };
    assert_eq!(plan.steps, [write]);
    assert_eq!(actions(&plan.items), ["skipped", "link skipped", "written"]);
    assert_eq!(plan.next, "add /src to PATH");
    assert!(skill_text_of(&plan.steps).contains("`mortar status`"));
}

#[test]
fn a_foreign_mr_earlier_on_path_names_mortar_with_or_without_the_link() {
    for short_name in [false, true] {
        let mut facts = facts();
        facts.first_mr = Some((PathBuf::from("/usr/bin/mr"), false));
        facts.first_mr_at = Some(0);
        facts.bin_at = Some(1);
        let plan = plan_install(&facts, &options(short_name), &paths(), None, &[&HARNESSES[0]], None).unwrap();
        assert_eq!(plan.next, "mortar init, in the repository Mortar is to keep");
        assert!(skill_text_of(&plan.steps).contains("`mortar status`"));
    }
}

#[test]
fn nothing_to_do_has_no_step() {
    let mut facts = facts();
    facts.binary_at = BinaryAt::Same;
    facts.link_at = LinkAt::Link { target: PathBuf::from("/h/.local/bin/mortar"), mortar: true };
    let plan = plan_install(&facts, &options(false), &paths(), None, &[], None).unwrap();
    assert!(plan.steps.is_empty());
    assert_eq!(actions(&plan.items), ["unchanged", "unchanged"]);
}
