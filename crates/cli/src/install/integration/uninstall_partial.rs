//! Recovery after uninstall cannot remove a recorded folder.

use std::fs;
use super::super::uninstall_in;
use super::support::{assert_report, deny_writes, options, snapshot, Tree};

#[test]
fn folder_failure_preserves_record_and_repeat_finishes() {
    let tree = Tree::new();
    fs::create_dir(tree.env.home.join(".claude")).unwrap();
    let before = snapshot(&tree.env.home);
    let mut opts = options(); opts.yes = true;
    assert_report(&tree.install(&opts), 0, "installed");
    let restore = deny_writes(tree.env.home.join(".claude/skills"));
    let report = uninstall_in(&tree.env.home);
    assert_report(&report, 5, "failed");
    let folder = tree.env.home.join(".claude/skills/mortar");
    assert!(report.lines.iter().any(|line| line.contains(folder.to_str().unwrap())), "{report:?}");
    assert!(tree.env.home.join(".local/state/mortar/install.json").is_file());
    assert_eq!(report.next, "repeat the same command");
    drop(restore);
    assert_report(&uninstall_in(&tree.env.home), 0, "uninstalled");
    assert_eq!(snapshot(&tree.env.home), before);
}

#[test]
fn ancestor_failure_names_previously_left_unrecorded_folder_in_order() {
    let tree = Tree::new();
    fs::create_dir(tree.env.home.join(".claude")).unwrap();
    let mut opts = options(); opts.yes = true;
    assert_report(&tree.install(&opts), 0, "installed");
    let stranger = tree.env.home.join(".local/state/mortar/stranger");
    fs::write(&stranger, b"stranger's data").unwrap();
    // HOME denies writes, so `.local/state/mortar` and `.local/state` are tried and left as not empty, then `.local`
    // fails: removing it needs a write to HOME.
    let _restore = deny_writes(tree.env.home.clone());
    let report = uninstall_in(&tree.env.home);
    assert_report(&report, 5, "failed");
    assert!(!tree.env.home.join(".local/state/mortar/install.json").exists());
    assert_eq!(fs::read(&stranger).unwrap(), b"stranger's data");
    let left: Vec<_> = report.lines.iter()
        .filter(|line| line.starts_with("left, and no longer in install's record: "))
        .collect();
    assert_eq!(left.len(), 3, "{report:?}");
    for (line, path) in left.iter().zip([".local/state/mortar", ".local/state", ".local"]) {
        assert!(line.ends_with(tree.env.home.join(path).to_str().unwrap()), "{report:?}");
    }
}

#[test]
fn ancestor_failure_names_stranger_bin_and_record_folders_in_order() {
    let tree = Tree::new();
    fs::create_dir(tree.env.home.join(".claude")).unwrap();
    let mut opts = options(); opts.yes = true;
    assert_report(&tree.install(&opts), 0, "installed");
    let stranger = tree.env.home.join(".local/bin/stranger");
    fs::write(&stranger, b"stranger's data").unwrap();
    let _restore = deny_writes(tree.env.home.join(".local/state"));
    let report = uninstall_in(&tree.env.home);
    assert_report(&report, 5, "failed");
    assert!(!tree.env.home.join(".local/state/mortar/install.json").exists());
    assert_eq!(fs::read(&stranger).unwrap(), b"stranger's data");
    let left: Vec<_> = report.lines.iter()
        .filter(|line| line.starts_with("left, and no longer in install's record: "))
        .collect();
    assert_eq!(left.len(), 4, "{report:?}");
    for (line, path) in left.iter().zip([".local/bin", ".local/state/mortar", ".local/state", ".local"]) {
        assert!(line.ends_with(tree.env.home.join(path).to_str().unwrap()), "{report:?}");
    }
    assert!(report.next.starts_with("remove each folder listed"), "{report:?}");
}

#[test]
fn ancestor_failure_names_all_unrecorded_remaining_folders() {
    let tree = Tree::new();
    fs::create_dir(tree.env.home.join(".claude")).unwrap();
    let mut opts = options(); opts.yes = true;
    assert_report(&tree.install(&opts), 0, "installed");
    let _restore = deny_writes(tree.env.home.join(".local/state"));
    let report = uninstall_in(&tree.env.home);
    assert_report(&report, 5, "failed");
    assert!(!tree.env.home.join(".local/state/mortar/install.json").exists());
    for path in [".local/state/mortar", ".local/state", ".local"] {
        let path = tree.env.home.join(path);
        assert!(report.lines.iter().any(|line| line.starts_with("left, and no longer in install's record") && line.ends_with(path.to_str().unwrap())), "{report:?}");
    }
    assert!(!report.next.contains("repeat the same command"), "{report:?}");
    assert!(report.next.starts_with("remove each folder listed"), "{report:?}");
}
