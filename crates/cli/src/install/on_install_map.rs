//! on_install_map: Whether a record item names a path of install's own map: the binary, the short-name link,
//! the skill files of item 5's table, and folders of the shapes install creates. Uninstall touches no file or link off
//! it. Folders are judged by shape, so a tampered record can name such a folder that install did not create; that can
//! only ever cost an empty folder, since uninstall removes a folder only when empty. A real install never records a
//! folder that already existed (item 7).

use std::path::{Path, PathBuf};

use super::{install_paths_for, skill_path, RecordItem, SkillLocation, HARNESSES};

/// True for a File at the placed binary under `home`, a Link at the `mr` link under `home`, a File ending in a
/// harness's `<skills>/mortar/SKILL.md` (global or local, so under any folder), and a Dir of a shape install creates:
/// `.local`, `.local/bin`, `.local/state` or `.local/state/mortar` strictly under `home`, or a path anywhere ending in
/// a leading part of `<skills>/mortar` (`.claude`, `.claude/skills`, `.claude/skills/mortar`, and so on), never
/// `home` itself. False for anything else, including an item whose kind does not match its path. Paths compare by
/// components, as the record holds them.
pub fn on_install_map(item: &RecordItem, home: &Path) -> bool {
    let paths = install_paths_for(home);
    match item {
        RecordItem::File { path, .. } => *path == paths.mortar || skill_tails().any(|tail| path.ends_with(tail)),
        RecordItem::Link { path, .. } => *path == paths.mr,
        // bin_dir, state_dir or a folder above them strictly under HOME (`.local`, `.local/bin`, `.local/state`,
        // `.local/state/mortar`), or a folder shaped like one on the way to a skill file.
        RecordItem::Dir { path } => {
            let fixed = [&paths.bin_dir, &paths.state_dir].iter().any(|dir| dir.starts_with(path));
            path != home
                && ((path.starts_with(home) && fixed)
                    || skill_tails().any(|tail| tail.ancestors().skip(1).any(|part| ends_with_part(path, part))))
        }
    }
}

/// Each harness's skill file relative to its base: `<skills>/mortar/SKILL.md`. skill_path with an empty base gives
/// the shape, so the shape is spelled only there; the base is HOME or any project, so the tail alone decides.
fn skill_tails() -> impl Iterator<Item = PathBuf> {
    HARNESSES.iter().map(|harness| skill_path(harness, SkillLocation::Global, Path::new(""), Path::new("")))
}

/// A folder ends with a non-empty leading part of a skill file's folders (`.claude`, `.claude/skills`,
/// `.claude/skills/mortar`); the empty part would match every folder.
fn ends_with_part(path: &Path, part: &Path) -> bool {
    !part.as_os_str().is_empty() && path.ends_with(part)
}
