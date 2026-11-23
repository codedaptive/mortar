//! set_fork_point: where a request cut with partners forks, for the open requests status (read_overlaps) and the
//! overlap and after-publish verbs (read_named_zones) read: the recorded base, while it is an ancestor of the tip.
//! No unit test: it runs git and reads the folder's recorded base. The integration tests
//! crates/cli/tests/status_need_runs/status_partners_tests.rs cover, with the mortar binary in request folders made
//! by `mr folder request`, plain and with --with, in a throwaway repository: a request cut with a partner, whose
//! partner's changes are counted as its own neither by status's overlaps nor by the overlap verb.

use std::path::Path;

use folders::{read_recorded_base, FolderKind, FolderRecord};
use gitio::{Git, Oid};

/// The recorded base (read_recorded_base on `git_dir`) of a Request record of `zone` whose recorded base has partners
/// and is an ancestor of `tip`: such a set member forks at the merge of its zone and the partners it was cut on, so
/// the partners' changes are not counted as its own. None for any other record, a base without partners, or a base
/// `tip` no longer holds; the caller then forks at its merge base with the zone. A failed read is its error text.
pub(crate) fn set_fork_point(
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
