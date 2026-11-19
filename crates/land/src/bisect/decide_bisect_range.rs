//! decide_bisect_range: bisect's two refusals before any probe, and the commits it searches when neither applies.
//! Pure: the first-parent reads come through `line`, so decide_bisect_range_tests drive it with a fake.

use gitio::Oid;

use crate::LandRefs;

use super::BisectError;

/// The commits after `last_pass` up to `failed`, oldest first, when both ends are on origin's first-parent line.
///
/// `line(base, tip)` is `tip`'s first-parent commits that `base` cannot reach, oldest first, when `base` is on that
/// line; else None. It is asked first for (`failed`, `tip`): None refuses with NotOnOrigin and nothing more is
/// asked, since a failed commit off origin names no landing. Then for (`last_pass`, `failed`): None refuses with
/// OffTheLine; Some is the answer. An Err from `line` is returned as it is.
pub(crate) fn decide_bisect_range(
    refs: &LandRefs,
    tip: &Oid,
    last_pass: &Oid,
    failed: &Oid,
    mut line: impl FnMut(&Oid, &Oid) -> Result<Option<Vec<Oid>>, BisectError>,
) -> Result<Vec<Oid>, BisectError> {
    if line(failed, tip)?.is_none() {
        return Err(BisectError::NotOnOrigin { failed: failed.clone(), origin_branch: refs.origin_branch.clone() });
    }
    line(last_pass, failed)?
        .ok_or_else(|| BisectError::OffTheLine { last_pass: last_pass.clone(), failed: failed.clone() })
}
