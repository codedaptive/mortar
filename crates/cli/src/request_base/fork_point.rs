//! fork_point: where a request forks, for the open requests status (status's read_overlaps) and the overlap and
//! after-publish verbs (after_publish's read_named_zones) read: the recorded base of a request cut with partners
//! while it is an ancestor of the tip, else the merge base of the tip and the zone tip.
//! No unit test: it runs git and reads the folder's recorded base. The integration tests
//! crates/cli/tests/status_need_runs/status_partners_tests.rs cover, with the mortar binary in request folders made
//! by `mr folder request`, plain and with --with, in a throwaway repository: a request cut with a partner, whose
//! partner's changes are counted as its own neither by status's overlaps nor by the overlap verb.

use std::path::Path;

use folders::{read_recorded_base, FolderKind, FolderRecord};
use gitio::{Git, GitError, Oid};

/// Where the request at `tip` forks from `zone`: set_fork_point's recorded base for its `folder` (its record and git
/// dir) when it gives one, else the merge base of `tip` and the zone tip. `zone_tip` is called only for that merge
/// base, so a caller may read the zone tip lazily; its error is returned as it is. No merge base is
/// `no_fork(None)`, a failed merge base `no_fork(Some(error))`: each caller words both. A failed read of the recorded
/// base is its error text.
pub(crate) fn fork_point(
    git: &Git,
    folder: Option<(&FolderRecord, &Path)>,
    zone: &str,
    tip: &Oid,
    zone_tip: impl FnOnce() -> Result<Oid, String>,
    no_fork: impl FnOnce(Option<GitError>) -> String,
) -> Result<Oid, String> {
    if let Some((record, git_dir)) = folder {
        if let Some(base) = set_fork_point(git, git_dir, record, zone, tip)? {
            return Ok(base);
        }
    }
    match git.merge_base(tip, &zone_tip()?) {
        Ok(Some(base)) => Ok(base),
        Ok(None) => Err(no_fork(None)),
        Err(error) => Err(no_fork(Some(error))),
    }
}

/// The recorded base (read_recorded_base on `git_dir`) of a Request record of `zone` whose recorded base has partners
/// and is an ancestor of `tip`: such a set member forks at the merge of its zone and the partners it was cut on, so
/// the partners' changes are not counted as its own. None for any other record, a base without partners, or a base
/// `tip` no longer holds.
fn set_fork_point(
    git: &Git,
    git_dir: &Path,
    record: &FolderRecord,
    zone: &str,
    tip: &Oid,
) -> Result<Option<Oid>, String> {
    if record.kind != FolderKind::Request || record.zone != zone {
        return Ok(None);
    }
    let Some(recorded) = read_recorded_base(git_dir)
        .map_err(|error| error.to_string())?
        .filter(|recorded| !recorded.partners.is_empty())
    else {
        return Ok(None);
    };
    Ok(git.is_ancestor(&recorded.base, tip).map_err(|error| error.to_string())?.then_some(recorded.base))
}
