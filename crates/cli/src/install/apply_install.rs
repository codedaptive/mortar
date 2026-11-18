//! apply_install: Applies install's steps in order, stopping at the first error (spec 07 item 7). It runs only once
//! install_in has found every refusal, so each step it applies was already allowed.
//! No sibling test: it touches the disk; the install ZITs in install/integration/ cover it (a fresh install, folders
//! that existed or not, install then uninstall leaving HOME as it was).

use std::path::Path;

use super::{apply_step, make_dirs, random_token, step_record, InstallStep, RecordItem};

/// For each step: make its folder and the missing ones above it (each made folder a Dir item, also those made before
/// a folder failed), apply_step with random_token, then step_record of it (`binary_hash` for a placed binary). The
/// items of what was applied and made, in order, and the first error ("cannot <place|link|write> <path>: <error>");
/// no step after it runs.
pub fn apply_install(steps: &[InstallStep], binary_hash: &str) -> (Vec<RecordItem>, Result<(), String>) {
    let mut items = Vec::new();
    for step in steps {
        let (verb, path) = match step {
            InstallStep::PlaceBinary { to, .. } => ("place", to),
            InstallStep::Link { at, .. } => ("link", at),
            InstallStep::WriteSkill { path, .. } => ("write", path),
        };
        let (made, result) = make_dirs(path.parent().unwrap_or(Path::new("")));
        items.extend(made.into_iter().map(|path| RecordItem::Dir { path }));
        let applied = result
            .map_err(|error| format!("cannot {verb} {}: {error}", path.display()))
            .and_then(|()| apply_step(step, &mut random_token));
        if let Err(error) = applied {
            return (items, Err(error));
        }
        items.push(step_record(step, binary_hash));
    }
    (items, Ok(()))
}
