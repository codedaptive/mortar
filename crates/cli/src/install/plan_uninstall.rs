//! plan_uninstall: What uninstall removes, from the record and what stands at each item now (spec 07, "mr uninstall",
//! S-177: a folder something else now uses stays).

use std::path::{Path, PathBuf};

use super::{on_install_map, InstallItem, ItemState, RecordItem, UninstallPlan};

/// An item not on install's map under `home` (on_install_map) is skipped and named, whatever its state: a record
/// cannot make uninstall remove anything else. Of the rest, files and links still as install placed them are
/// removed; changed ones are left and named. Folders that are still folders go to `dirs`, deepest first, with no
/// item; a gone folder has no item; a folder path that now holds anything else is left and named. `states` is in
/// record order.
pub fn plan_uninstall(record: &[RecordItem], states: &[ItemState], home: &Path) -> UninstallPlan {
    let mut plan = UninstallPlan { remove: Vec::new(), dirs: Vec::new(), items: Vec::new() };
    for (item, state) in record.iter().zip(states) {
        match fate(item, state, home) {
            Fate::Folder(path) => plan.dirs.push(path.clone()),
            Fate::Nothing => {}
            Fate::Named(path, action, note) => {
                if action == "removed" {
                    plan.remove.push(path.clone());
                }
                plan.items.push(InstallItem { action: action.to_string(), path: path.clone(), note: note.to_string() });
            }
        }
    }
    // Deepest first, so a folder's subfolders are tried before it; reverse path order breaks ties.
    plan.dirs.sort_by(|a, b| b.components().count().cmp(&a.components().count()).then_with(|| b.cmp(a)));
    plan
}

/// What becomes of one record item: a folder to try, nothing (a gone folder), or a report line (action, note).
enum Fate<'a> {
    Folder(&'a PathBuf),
    Nothing,
    Named(&'a PathBuf, &'static str, &'static str),
}

fn fate<'a>(item: &'a RecordItem, state: &ItemState, home: &Path) -> Fate<'a> {
    let (RecordItem::File { path, .. } | RecordItem::Link { path, .. } | RecordItem::Dir { path }) = item;
    if !on_install_map(item, home) {
        return Fate::Named(path, "skipped", "not a path install places");
    }
    match (item, state) {
        (RecordItem::Dir { .. }, ItemState::Dir) => Fate::Folder(path),
        (RecordItem::Dir { .. }, ItemState::Gone) => Fate::Nothing,
        (RecordItem::Dir { .. }, _) => Fate::Named(path, "left", "no longer a folder"),
        (RecordItem::File { hash, .. }, ItemState::File { hash: now }) if now == hash => {
            Fate::Named(path, "removed", "")
        }
        (RecordItem::Link { target, .. }, ItemState::Link { target: now }) if now == target => {
            Fate::Named(path, "removed", "")
        }
        (_, ItemState::Gone) => Fate::Named(path, "gone", ""),
        _ => Fate::Named(path, "left", "changed since install"),
    }
}
