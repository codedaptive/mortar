//! The attempts a keep or a drop would discard without the caller's saved mark (S-32).

use std::path::PathBuf;

use super::Attempt;

/// In k order, every attempt in `attempts` other than attempt `except` that is not listed in `saved` as (its k,
/// true), each named by its folder, or by its full branch name as a path when it has no folder, so that
/// AttemptsUnsaved needs no second variant.
pub(crate) fn unsaved_attempts(attempts: &[Attempt], saved: &[(u32, bool)], except: Option<u32>) -> Vec<PathBuf> {
    let mut sorted: Vec<&Attempt> = attempts.iter().collect();
    sorted.sort_by_key(|a| a.k);
    sorted
        .into_iter()
        .filter(|a| Some(a.k) != except && !saved.contains(&(a.k, true)))
        .map(|a| a.folder.clone().unwrap_or_else(|| PathBuf::from(&a.branch)))
        .collect()
}
