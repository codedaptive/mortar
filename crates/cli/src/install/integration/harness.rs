//! Harness selection and the command named in installed skills.

use std::fs;

use super::super::{install_in, SkillLocation, HARNESSES};
use super::support::{assert_report, executable, options, snapshot, Tree};

#[test]
fn clash_skips_link_and_skill_names_mortar() {
    let mut tree = Tree::new();
    fs::create_dir(tree.env.home.join(".claude")).unwrap();
    let foreign = tree.root.join("foreign/mr");
    executable(&foreign, b"another program");
    let bin = tree.env.home.join(".local/bin");
    tree.path(&[foreign.parent().unwrap(), &bin]);
    let mut opts = options();
    opts.yes = true;
    let report = tree.install(&opts);
    assert_report(&report, 0, "installed");
    assert!(!bin.join("mr").exists());
    assert!(report.lines.iter().any(|line| line.contains(foreign.to_str().unwrap())));
    let skill = fs::read_to_string(tree.env.home.join(".claude/skills/mortar/SKILL.md")).unwrap();
    assert!(skill.contains("mortar status"));
    assert!(!skill.contains("`mr status`"));
}

#[test]
fn forced_link_behind_clash_still_names_mortar() {
    let mut tree = Tree::new();
    fs::create_dir(tree.env.home.join(".claude")).unwrap();
    let foreign = tree.root.join("foreign/mr");
    executable(&foreign, b"another program");
    let bin = tree.env.home.join(".local/bin");
    tree.path(&[foreign.parent().unwrap(), &bin]);
    let mut opts = options();
    opts.yes = true;
    opts.short_name = true;
    assert_report(&tree.install(&opts), 0, "installed");
    assert!(fs::symlink_metadata(bin.join("mr")).unwrap().file_type().is_symlink());
    assert_eq!(fs::read_link(bin.join("mr")).unwrap(), bin.join("mortar"));
    let skill = fs::read_to_string(tree.env.home.join(".claude/skills/mortar/SKILL.md")).unwrap();
    assert!(skill.contains("mortar status"));
    assert!(!skill.contains("`mr status`"));
}

#[test]
fn first_link_on_path_makes_skill_name_mr() {
    let tree = Tree::new();
    fs::create_dir(tree.env.home.join(".claude")).unwrap();
    let mut opts = options();
    opts.yes = true;
    assert_report(&tree.install(&opts), 0, "installed");
    let skill = fs::read_to_string(tree.env.home.join(".claude/skills/mortar/SKILL.md")).unwrap();
    assert!(skill.contains("mr status"));
    assert!(!skill.contains("mortar status"));
}

#[test]
fn yes_wires_detected_harnesses_only() {
    let tree = Tree::new();
    fs::create_dir(tree.env.home.join(".claude")).unwrap();
    fs::create_dir(tree.env.home.join(".cursor")).unwrap();
    let mut opts = options();
    opts.yes = true;
    assert_report(&tree.install(&opts), 0, "installed");
    assert!(tree.env.home.join(".claude/skills/mortar/SKILL.md").is_file());
    assert!(tree.env.home.join(".cursor/skills/mortar/SKILL.md").is_file());
    assert!(!tree.env.home.join(".agents").exists());
}

#[test]
fn undetected_target_refuses_without_writes() {
    let tree = Tree::new();
    fs::create_dir(tree.env.home.join(".claude")).unwrap();
    let before = snapshot(&tree.env.home);
    let mut opts = options();
    opts.targets = Some(vec![&HARNESSES[1]]);
    assert_report(&tree.install(&opts), 3, "refused");
    assert_eq!(snapshot(&tree.env.home), before);
}

#[test]
fn nonterminal_without_selection_explains_wiring() {
    let tree = Tree::new();
    fs::create_dir(tree.env.home.join(".claude")).unwrap();
    let report = tree.install(&options());
    assert_report(&report, 0, "installed");
    assert!(!tree.env.home.join(".claude/skills/mortar/SKILL.md").exists());
    assert!(report.lines.iter().any(|line| line.contains("--target") && line.contains("-y")), "{report:?}");
}

#[test]
fn local_target_writes_skill_under_project() {
    let tree = Tree::new();
    fs::create_dir(tree.env.home.join(".claude")).unwrap();
    let mut opts = options();
    opts.targets = Some(vec![&HARNESSES[0]]);
    opts.location = SkillLocation::Local;
    assert_report(&tree.install(&opts), 0, "installed");
    assert!(tree.env.project.join(".claude/skills/mortar/SKILL.md").is_file());
    assert!(!tree.env.home.join(".claude/skills/mortar/SKILL.md").exists());
}

#[test]
fn picker_selects_cursor_only() {
    let mut tree = Tree::new();
    fs::create_dir(tree.env.home.join(".claude")).unwrap();
    fs::create_dir(tree.env.home.join(".cursor")).unwrap();
    tree.env.terminal = true;
    let mut prompts = Vec::new();
    let report = install_in(&tree.env, &options(), &mut |prompt| {
        prompts.push(prompt.to_owned());
        Some("cursor".to_owned())
    });
    assert_report(&report, 0, "installed");
    assert_eq!(prompts.len(), 1);
    assert!(prompts[0].contains("claude") && prompts[0].contains("cursor"));
    assert!(tree.env.home.join(".cursor/skills/mortar/SKILL.md").is_file());
    assert!(!tree.env.home.join(".claude/skills/mortar/SKILL.md").exists());
}
