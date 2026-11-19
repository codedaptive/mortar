//! uninstall_in: `mr uninstall` under a given HOME: removes what install's record lists that is on install's map
//! (on_install_map) and still as install placed it. Files and links go first, then
//! each recorded folder that is still a folder and empty, deepest first, except the folders holding the record; then
//! the record; then the folders holding it, deepest first. A recorded folder that is not empty or is no longer a
//! folder is left and named. A record entry off the map is skipped and named, whatever stands there. Nothing else is
//! removed.
//! No sibling test: it touches the disk. The install ZITs in install/integration/uninstall.rs and
//! uninstall_partial.rs cover it.

use std::fs;
use std::io::{self, ErrorKind};
use std::path::{Path, PathBuf};

use crate::report::{ExitCode, Report};

use super::{
    install_paths_for, observe_item, plan_uninstall, read_record, uninstall_failure, uninstall_report, InstallItem,
    ItemState, UninstallPlan,
};

/// Uninstall's report, a refusal for an unreadable record, or a failure (exit 5) when the disk fails. A failure
/// before the record is removed leaves the record in place, so a repeat is safe and finds what is left (what was
/// removed reads as gone). A failure after it names the recorded folders left before the failure, the failed
/// folder and those not yet tried, which a repeat no longer sees.
pub fn uninstall_in(home: &Path) -> Report {
    let record_path = install_paths_for(home).record;
    let record = match read_record(&record_path) {
        Err(why) => return record_refusal(&why, &record_path),
        Ok(None) => return uninstall_report(&record_path, None),
        Ok(Some(record)) => record,
    };
    let states: Vec<ItemState> = record.iter().map(observe_item).collect();
    let mut plan = plan_uninstall(&record, &states, home);
    match remove(&plan, &record_path) {
        Ok(dir_items) => {
            plan.items.extend(dir_items);
            uninstall_report(&record_path, Some(&plan.items))
        }
        Err((error, left)) => uninstall_failure(&error, &left),
    }
}

/// Removes, in order: the plan's files and links; the folders of `dirs` that do not hold the record; the record; the
/// folders of `dirs` that hold it. Each stage keeps `dirs`' deepest-first order, and a folder's parent holds the
/// record whenever the folder does, so every folder is still tried after the ones inside it. The folders' items
/// ("removed", or "left" when not empty or no longer a folder; none for a folder already gone). A record already
/// gone (removed after it was read) counts as removed. On an error, the folders that install's record no longer
/// lists and are still on disk: none while the record is in place; after it is gone, the folders that do not hold
/// the record and were left (in the order tried), then the record-holding folders already tried and left, the failed
/// folder, and those not yet tried. Every folder still comes before the folders holding it: a folder holding a
/// record-holding folder holds the record too, so none of the first group holds one of the second.
fn remove(plan: &UninstallPlan, record_path: &Path) -> Result<Vec<InstallItem>, (String, Vec<PathBuf>)> {
    let kept = |error: String| (error, Vec::new());
    for path in &plan.remove {
        fs::remove_file(path).map_err(|error| kept(cannot(path, &error)))?;
    }
    let (holding, others): (Vec<&PathBuf>, Vec<&PathBuf>) =
        plan.dirs.iter().partition(|dir| record_path.starts_with(dir));
    let mut items = Vec::new();
    let mut left = Vec::new();
    for dir in others {
        let item = remove_dir(dir).map_err(kept)?;
        left.extend(left_path(&item));
        items.extend(item);
    }
    match fs::remove_file(record_path) {
        Err(error) if error.kind() != ErrorKind::NotFound => return Err(kept(cannot(record_path, &error))),
        _ => {}
    }
    for (tried, dir) in holding.iter().enumerate() {
        let item = remove_dir(dir).map_err(|error| {
            (error, left.iter().cloned().chain(holding[tried..].iter().map(|dir| dir.to_path_buf())).collect())
        })?;
        left.extend(left_path(&item));
        items.extend(item);
    }
    Ok(items)
}

/// The folder an item names when it was left on disk.
fn left_path(item: &Option<InstallItem>) -> Option<PathBuf> {
    item.as_ref().filter(|item| item.action == "left").map(|item| item.path.clone())
}

/// Removes `dir` if empty: its item, None when it is already gone, or Err for any other error.
fn remove_dir(dir: &Path) -> Result<Option<InstallItem>, String> {
    let (action, note) = match fs::remove_dir(dir) {
        Ok(()) => ("removed", ""),
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(None),
        Err(error) if error.kind() == ErrorKind::DirectoryNotEmpty => ("left", "not empty"),
        // Became something else between plan_uninstall's look and now.
        Err(error) if error.kind() == ErrorKind::NotADirectory => ("left", "no longer a folder"),
        Err(error) => return Err(cannot(dir, &error)),
    };
    Ok(Some(InstallItem { action: action.to_string(), path: dir.to_path_buf(), note: note.to_string() }))
}

fn cannot(path: &Path, error: &io::Error) -> String {
    format!("cannot remove {}: {error}", path.display())
}

/// Refused (exit 1): the record cannot be read or decoded, so uninstall cannot tell what is install's.
fn record_refusal(why: &str, record_path: &Path) -> Report {
    Report {
        verb: "uninstall".to_string(),
        exit: ExitCode::Refused,
        outcome: "refused".to_string(),
        lines: vec![why.to_string(), "nothing changed".to_string()],
        owner: Some("the person uninstalling Mortar".to_string()),
        next: format!("fix or remove {}", record_path.display()),
        fields: Vec::new(),
    }
}
