//! Recovery after a write fails after binary placement.

use std::fs;
use super::super::{apply_failure, apply_install, uninstall_in, InstallStep, RecordItem};
use super::support::{assert_report, deny_writes, executable, options, snapshot, RestorePermissions, Tree};

fn fail_skill_write(tree: &Tree) -> RestorePermissions {
    let dir = tree.env.home.join(".claude/skills/mortar");
    fs::create_dir_all(&dir).unwrap();
    deny_writes(dir)
}

#[test]
fn failed_install_names_recorded_placement_and_recovery() {
    let tree = Tree::new();
    let _restore = fail_skill_write(&tree);
    let mut opts = options(); opts.yes = true;
    let report = tree.install(&opts);
    assert_report(&report, 5, "failed");
    for path in [".local/bin/mortar", ".local/bin/mr"] {
        let path = tree.env.home.join(path);
        assert!(report.lines.iter().any(|line| line.ends_with(path.to_str().unwrap()) && line.contains("mr uninstall")), "{report:?}");
    }
    assert!(!report.lines.iter().any(|line| line == "nothing half-done is left"));
    assert!(report.next.contains("repeat the same command") && report.next.contains("mr uninstall"), "{report:?}");
}

#[test]
fn foreign_mr_partial_install_names_mortar_for_recovery() {
    let mut tree = Tree::new();
    let foreign = tree.root.join("foreign/mr");
    executable(&foreign, b"another program");
    let bin = tree.env.home.join(".local/bin");
    tree.path(&[foreign.parent().unwrap(), &bin]);
    let _restore = fail_skill_write(&tree);
    let mut opts = options(); opts.yes = true;
    let report = tree.install(&opts);
    assert_report(&report, 5, "failed");
    assert_eq!(fs::read(&foreign).unwrap(), b"another program");
    assert!(fs::symlink_metadata(bin.join("mr")).is_err(), "the mr link was made: {report:?}");
    assert!(report.lines.iter().any(|line| line.ends_with(bin.join("mortar").to_str().unwrap()) && line.contains("mortar uninstall")), "{report:?}");
    assert!(!report.lines.iter().any(|line| line.contains("mr uninstall")), "{report:?}");
    assert!(!report.next.contains("mr uninstall"), "{report:?}");
    assert!(report.next.contains("repeat the same command") && report.next.contains("mortar uninstall"), "{report:?}");
}

#[test]
fn failed_link_after_placed_binary_names_mortar_for_recovery() {
    let tree = Tree::new();
    let binary = tree.env.home.join(".local/bin/mortar");
    let denied = tree.root.join("denied");
    fs::create_dir(&denied).unwrap();
    let _restore = deny_writes(denied.clone());
    let link = denied.join("missing/mr");
    let steps = [
        InstallStep::PlaceBinary { from: tree.env.exe.clone().unwrap(), to: binary.clone() },
        InstallStep::Link { at: link.clone(), target: binary.clone() },
    ];
    let (items, result) = apply_install(&steps, "hash");
    let error = result.unwrap_err();
    assert!(error.starts_with(&format!("cannot link {}:", link.display())), "{error}");
    assert!(items.contains(&RecordItem::File { path: binary.clone(), hash: "hash".into() }), "{items:?}");
    assert!(!items.iter().any(|item| matches!(item, RecordItem::Link { .. })), "{items:?}");
    let report = apply_failure(&error, &items, "mr");
    assert!(report.lines.iter().any(|line| line.ends_with(binary.to_str().unwrap()) && line.contains("mortar uninstall")), "{report:?}");
    assert!(!report.lines.iter().any(|line| line.contains("mr uninstall")), "{report:?}");
    assert!(!report.next.contains("mr uninstall"), "{report:?}");
    assert!(report.next.contains("mortar uninstall"), "{report:?}");
}

#[test]
fn repeat_finishes_partial_install() {
    let tree = Tree::new();
    let restore = fail_skill_write(&tree);
    let mut opts = options(); opts.yes = true;
    assert_report(&tree.install(&opts), 5, "failed");
    drop(restore);
    assert_report(&tree.install(&opts), 0, "installed");
    assert!(tree.env.home.join(".claude/skills/mortar/SKILL.md").is_file());
}

#[test]
fn uninstall_removes_partial_install() {
    let tree = Tree::new();
    let restore = fail_skill_write(&tree);
    let before = snapshot(&tree.env.home);
    let mut opts = options(); opts.yes = true;
    assert_report(&tree.install(&opts), 5, "failed");
    drop(restore);
    assert_report(&uninstall_in(&tree.env.home), 0, "uninstalled");
    assert_eq!(snapshot(&tree.env.home), before);
}
