//! cap_refusals: which changed files the size cap leaves out of a snapshot.

use crate::cap_candidate::CapCandidate;
use crate::refused_file::RefusedFile;

/// Each candidate not in HEAD whose size is over `cap`, in input order, carrying `cap`. Over means size > cap, so a
/// file of exactly the cap is kept. Sizes and cap are bytes; the caller's default is DEFAULT_SIZE_CAP,
/// 5 MiB = 5 * 1024 * 1024. A file already in HEAD is kept at any size: the cap stops new large files only.
pub(crate) fn cap_refusals(candidates: &[CapCandidate], cap: u64) -> Vec<RefusedFile> {
    candidates
        .iter()
        .filter(|c| !c.in_head && c.size > cap)
        .map(|c| RefusedFile { path: c.path.clone(), size: c.size, cap })
        .collect()
}
