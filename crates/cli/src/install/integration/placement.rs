//! Placement, links and repeat installs on an isolated PATH.

use std::fs;
use std::os::unix::fs::{MetadataExt, PermissionsExt};

use super::support::{assert_report, executable, options, snapshot, Tree};

#[test]
fn fresh_install_copies_binary_and_links_short_name() {
    let tree = Tree::new();
    let report = tree.install(&options());
    assert_report(&report, 0, "installed");
    let binary = tree.env.home.join(".local/bin/mortar");
    let link = tree.env.home.join(".local/bin/mr");
    let meta = fs::symlink_metadata(&binary).unwrap();
    assert!(meta.file_type().is_file());
    assert_eq!(meta.permissions().mode() & 0o777, 0o755);
    assert_eq!(meta.nlink(), 1);
    assert_eq!(fs::read(&binary).unwrap(), fs::read(tree.env.exe.as_ref().unwrap()).unwrap());
    assert!(fs::symlink_metadata(&link).unwrap().file_type().is_symlink());
    assert_eq!(fs::read_link(&link).unwrap(), binary);
    assert!(tree.env.home.join(".local/state/mortar/install.json").is_file());
}

#[test]
fn repeat_install_is_unchanged() {
    let tree = Tree::new();
    assert_report(&tree.install(&options()), 0, "installed");
    let before = snapshot(&tree.env.home);
    assert_report(&tree.install(&options()), 0, "already-installed");
    assert_eq!(snapshot(&tree.env.home), before);
}

#[test]
fn prefix_binary_is_not_placed_and_link_targets_path_name() {
    let mut tree = Tree::new();
    let bin = tree.env.prefixes[0].join("bin");
    let exe = bin.join("mortar");
    executable(&exe, &fs::read(tree.env.exe.as_ref().unwrap()).unwrap());
    tree.env.exe = Some(exe.clone());
    tree.path(&[&bin]);
    assert_report(&tree.install(&options()), 0, "installed");
    assert!(!tree.env.home.join(".local/bin/mortar").exists());
    let link = tree.env.home.join(".local/bin/mr");
    assert!(fs::symlink_metadata(&link).unwrap().file_type().is_symlink());
    assert_eq!(fs::read_link(link).unwrap(), exe);
}

#[test]
fn no_place_skips_copy_for_non_prefix_binary() {
    let tree = Tree::new();
    let mut opts = options();
    opts.no_place = true;
    let report = tree.install(&opts);
    assert_eq!(report.exit as i32, 0);
    assert!(!tree.env.home.join(".local/bin/mortar").exists());
}

#[test]
fn skipped_placement_without_mortar_on_path_has_no_link_and_says_how() {
    let tree = Tree::new();
    let mut opts = options();
    opts.no_place = true;
    let report = tree.install(&opts);
    assert_eq!(report.exit as i32, 0);
    assert!(!tree.env.home.join(".local/bin/mr").exists());
    assert!(report.next.contains("PATH"), "{report:?}");
    assert!(report.next.contains(tree.root.join("source").to_str().unwrap()), "{report:?}");
}
