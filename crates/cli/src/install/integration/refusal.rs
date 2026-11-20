//! Refusals and the foreign short-name clash exception.

use std::fs;

use super::support::{assert_report, deny_writes, executable, options, snapshot, Tree};

#[test]
fn a_file_where_the_state_folder_goes_is_refused_before_any_write() {
    let tree = Tree::new();
    fs::create_dir(tree.env.home.join(".claude")).unwrap();
    fs::create_dir(tree.env.home.join(".local")).unwrap();
    let state = tree.env.home.join(".local/state");
    fs::write(&state, b"not a directory").unwrap();
    let before = snapshot(&tree.env.home);
    let mut opts = options();
    opts.yes = true;
    let report = tree.install(&opts);
    assert_report(&report, 1, "refused");
    assert!(report.lines.iter().any(|line| line.contains(state.to_str().unwrap())), "{report:?}");
    assert_eq!(snapshot(&tree.env.home), before);
}

#[test]
fn a_state_folder_that_cannot_be_made_places_nothing() {
    let tree = Tree::new();
    fs::create_dir(tree.env.home.join(".claude")).unwrap();
    let state = tree.env.home.join(".local/state");
    fs::create_dir_all(&state).unwrap();
    let _restore = deny_writes(state.clone());
    let before = snapshot(&tree.env.home);
    let mut opts = options();
    opts.yes = true;
    let report = tree.install(&opts);
    assert_report(&report, 5, "failed");
    let state_dir = state.join("mortar");
    assert!(
        report.lines.iter().any(|line| line.contains("cannot make") && line.contains(state_dir.to_str().unwrap())),
        "{report:?}"
    );
    assert_eq!(snapshot(&tree.env.home), before);
    assert!(!tree.env.home.join(".local/bin").exists());
}

#[test]
fn foreign_short_name_on_path_is_left_when_link_skipped() {
    let tree = Tree::new();
    let foreign = tree.env.home.join(".local/bin/mr");
    executable(&foreign, b"foreign mr");
    let report = tree.install(&options());
    assert_report(&report, 0, "installed");
    assert_eq!(fs::read(&foreign).unwrap(), b"foreign mr");
    assert!(report.lines.iter().any(|line| line.contains(foreign.to_str().unwrap())));
}

#[test]
fn forcing_link_over_foreign_short_name_refuses_without_writes() {
    let tree = Tree::new();
    executable(&tree.env.home.join(".local/bin/mr"), b"foreign mr");
    let before = snapshot(&tree.env.home);
    let mut opts = options();
    opts.short_name = true;
    assert_report(&tree.install(&opts), 1, "refused");
    assert_eq!(snapshot(&tree.env.home), before);
}

#[test]
fn foreign_binary_refuses_without_writes() {
    let tree = Tree::new();
    executable(&tree.env.home.join(".local/bin/mortar"), b"foreign mortar");
    let before = snapshot(&tree.env.home);
    assert_report(&tree.install(&options()), 1, "refused");
    assert_eq!(snapshot(&tree.env.home), before);
}

#[test]
fn foreign_skill_refuses_without_writes() {
    let tree = Tree::new();
    let skill = tree.env.home.join(".claude/skills/mortar/SKILL.md");
    fs::create_dir_all(skill.parent().unwrap()).unwrap();
    fs::write(&skill, b"someone else's skill").unwrap();
    let before = snapshot(&tree.env.home);
    let mut opts = options();
    opts.yes = true;
    assert_report(&tree.install(&opts), 1, "refused");
    assert_eq!(snapshot(&tree.env.home), before);
}
