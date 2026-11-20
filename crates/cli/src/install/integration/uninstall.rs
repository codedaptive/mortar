//! Removal of recorded files and install-created folders on install's map; record entries off it are skipped.

use std::fs;

use super::super::{content_hash, decode_record, encode_record, uninstall_in, RecordItem};
use super::support::{assert_report, options, snapshot, Tree};

#[test]
fn uninstall_restores_home_after_install() {
    let tree = Tree::new();
    fs::create_dir(tree.env.home.join(".claude")).unwrap();
    let before = snapshot(&tree.env.home);
    let mut opts = options();
    opts.yes = true;
    assert_report(&tree.install(&opts), 0, "installed");
    assert_report(&uninstall_in(&tree.env.home), 0, "uninstalled");
    assert_eq!(snapshot(&tree.env.home), before);
}

#[test]
fn uninstall_keeps_preexisting_directory() {
    let tree = Tree::new();
    let bin = tree.env.home.join(".local/bin");
    fs::create_dir_all(&bin).unwrap();
    assert_report(&tree.install(&options()), 0, "installed");
    assert_report(&uninstall_in(&tree.env.home), 0, "uninstalled");
    assert!(bin.is_dir());
    assert!(fs::read_dir(bin).unwrap().next().is_none());
}

#[test]
fn uninstall_keeps_created_directory_that_is_no_longer_empty() {
    let tree = Tree::new();
    assert_report(&tree.install(&options()), 0, "installed");
    let stranger = tree.env.home.join(".local/bin/stranger");
    fs::write(&stranger, b"not Mortar's").unwrap();
    assert_report(&uninstall_in(&tree.env.home), 0, "uninstalled");
    assert_eq!(fs::read(stranger).unwrap(), b"not Mortar's");
    assert!(tree.env.home.join(".local/bin").is_dir());
}

#[test]
fn uninstall_keeps_and_names_edited_skill_but_removes_binary_and_link() {
    let tree = Tree::new();
    fs::create_dir(tree.env.home.join(".claude")).unwrap();
    let mut opts = options();
    opts.yes = true;
    assert_report(&tree.install(&opts), 0, "installed");
    let skill = tree.env.home.join(".claude/skills/mortar/SKILL.md");
    fs::write(&skill, b"edited by its owner").unwrap();
    let report = uninstall_in(&tree.env.home);
    assert_report(&report, 0, "uninstalled");
    assert_eq!(fs::read(&skill).unwrap(), b"edited by its owner");
    assert!(report.lines.iter().any(|line| line.contains(skill.to_str().unwrap())), "{report:?}");
    assert!(fs::symlink_metadata(tree.env.home.join(".local/bin/mortar")).is_err());
    assert!(fs::symlink_metadata(tree.env.home.join(".local/bin/mr")).is_err());
}

#[test]
fn uninstall_leaves_a_recorded_folder_that_became_a_file() {
    let tree = Tree::new();
    fs::create_dir(tree.env.home.join(".claude")).unwrap();
    let mut opts = options();
    opts.yes = true;
    assert_report(&tree.install(&opts), 0, "installed");

    let folder = tree.env.home.join(".claude/skills/mortar");
    fs::remove_file(folder.join("SKILL.md")).unwrap();
    fs::remove_dir(&folder).unwrap();
    fs::write(&folder, b"replacement file").unwrap();

    let report = uninstall_in(&tree.env.home);
    assert_eq!(report.exit as i32, 0, "{report:?}");
    assert!(report.lines.iter().any(|line| line.contains(folder.to_str().unwrap())), "{report:?}");
    assert!(fs::symlink_metadata(&folder).unwrap().file_type().is_file());
    assert_eq!(fs::read(&folder).unwrap(), b"replacement file");
    assert!(fs::symlink_metadata(tree.env.home.join(".local/bin/mortar")).is_err());
    assert!(fs::symlink_metadata(tree.env.home.join(".local/bin/mr")).is_err());
}

#[test]
fn uninstall_skips_entries_off_install_map() {
    let tree = Tree::new();
    fs::create_dir(tree.env.home.join(".claude")).unwrap();
    let mut opts = options();
    opts.yes = true;
    assert_report(&tree.install(&opts), 0, "installed");

    let notes = tree.env.home.join("notes.txt");
    let keep = tree.env.home.join("keep");
    fs::write(&notes, b"keep me").unwrap();
    fs::create_dir(&keep).unwrap();
    let record = tree.env.home.join(".local/state/mortar/install.json");
    let mut items = decode_record(&fs::read_to_string(&record).unwrap()).unwrap();
    items.push(RecordItem::File { path: notes.clone(), hash: content_hash(b"keep me") });
    items.push(RecordItem::Dir { path: keep.clone() });
    fs::write(&record, encode_record(&items).unwrap()).unwrap();

    let report = uninstall_in(&tree.env.home);
    assert_report(&report, 0, "uninstalled");
    assert!(notes.is_file(), "off-map file must survive uninstall");
    assert_eq!(fs::read(&notes).unwrap(), b"keep me");
    assert!(keep.is_dir(), "off-map directory must survive uninstall");
    for path in [&notes, &keep] {
        assert!(
            report.lines.iter().any(|line| line.contains("skipped") && line.contains(path.to_str().unwrap())),
            "{report:?}"
        );
    }
    for path in [".local/bin/mortar", ".local/bin/mr", ".claude/skills/mortar/SKILL.md"] {
        assert!(fs::symlink_metadata(tree.env.home.join(path)).is_err(), "{path} still exists");
    }
}

#[test]
fn uninstall_without_record_changes_nothing() {
    let tree = Tree::new();
    fs::write(tree.env.home.join("keep"), b"unrelated").unwrap();
    let before = snapshot(&tree.env.home);
    assert_report(&uninstall_in(&tree.env.home), 0, "not-installed");
    assert_eq!(snapshot(&tree.env.home), before);
}
