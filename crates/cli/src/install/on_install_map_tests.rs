//! on_install_map: the binary, the `mr` link, harness skill files and the folders on their way are on the map;
//! anything else, or a kind that does not match its path, is not. The test HOME is `/h`; a local skill sits under
//! the project `/p`.

use std::path::{Path, PathBuf};

use super::{on_install_map, RecordItem};

fn file(path: &str) -> RecordItem {
    RecordItem::File { path: PathBuf::from(path), hash: "aaaa".to_string() }
}

fn link(path: &str) -> RecordItem {
    RecordItem::Link { path: PathBuf::from(path), target: PathBuf::from("/h/.local/bin/mortar") }
}

fn dir(path: &str) -> RecordItem {
    RecordItem::Dir { path: PathBuf::from(path) }
}

fn on(item: &RecordItem) -> bool {
    on_install_map(item, Path::new("/h"))
}

#[test]
fn the_placed_binary_and_the_short_name_link_are_on_the_map() {
    assert!(on(&file("/h/.local/bin/mortar")));
    assert!(on(&link("/h/.local/bin/mr")));
}

#[test]
fn each_harness_skill_file_global_or_local_is_on_the_map() {
    for path in [
        "/h/.claude/skills/mortar/SKILL.md",
        "/p/.claude/skills/mortar/SKILL.md",
        "/h/.agents/skills/mortar/SKILL.md",
        "/p/.agents/skills/mortar/SKILL.md",
        "/h/.cursor/skills/mortar/SKILL.md",
        "/p/.cursor/skills/mortar/SKILL.md",
    ] {
        assert!(on(&file(path)), "{path}");
    }
}

#[test]
fn each_folder_install_may_create_is_on_the_map() {
    for path in [
        "/h/.local",
        "/h/.local/bin",
        "/h/.local/state",
        "/h/.local/state/mortar",
        "/h/.claude",
        "/h/.claude/skills",
        "/h/.claude/skills/mortar",
        "/p/.claude/skills/mortar",
        "/h/.agents",
        "/h/.agents/skills",
        "/h/.agents/skills/mortar",
        "/p/.agents",
        "/h/.cursor",
        "/h/.cursor/skills",
        "/h/.cursor/skills/mortar",
        "/p/.cursor/skills",
    ] {
        assert!(on(&dir(path)), "{path}");
    }
}

#[test]
fn a_kind_that_does_not_match_its_path_is_off_the_map() {
    assert!(!on(&link("/h/.local/bin/mortar")));
    assert!(!on(&file("/h/.local/bin/mr")));
    assert!(!on(&link("/h/.claude/skills/mortar/SKILL.md")));
    assert!(!on(&dir("/h/.local/bin/mortar")));
    assert!(!on(&dir("/h/.claude/skills/mortar/SKILL.md")));
}

#[test]
fn near_misses_are_off_the_map() {
    for item in [
        file("/h/.local/bin/other"),
        file("/x/.claude/skills/other/SKILL.md"),
        file("/h/.codex/skills/mortar/SKILL.md"),
        file("/h/notes.txt"),
        file("/x/.local/bin/mortar"),
        link("/x/.local/bin/mr"),
        dir("/h"),
        dir("/"),
        dir("/h/.local/share"),
        dir("/h/keep"),
        dir("/x/.local/bin"),
        dir("/h/.local/state/mortar/install.json"),
    ] {
        assert!(!on(&item), "{item:?}");
    }
}

#[test]
fn home_itself_is_never_on_the_map_even_when_shaped_like_a_skill_folder() {
    assert!(!on_install_map(&dir("/u/.claude"), Path::new("/u/.claude")));
}
