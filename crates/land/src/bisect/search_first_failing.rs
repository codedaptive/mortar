//! Mortar's own binary search for the first failing landing (S-61), over a probe the caller supplies.

use gitio::Oid;

use super::{BisectNotFound, BisectVerdict};

/// The commit of `candidates` that fails while the one before it passes.
///
/// `candidates` are the commits after `last_pass` up to and including the failed one, oldest first: origin's
/// first parents, one per landing (S-61); the last is the failed commit. `passes(c)` is one probe: Ok(true)
/// means the caller's command exits 0 at `c`.
///
/// The ends are checked before the search: with no candidates, `last_pass` is the failed commit and its one
/// probe says which NotFound applies; otherwise the failed commit is probed first (it passes: FailedPasses),
/// then `last_pass` (it fails: LastPassFails). The search then keeps a passing position and a failing one and
/// halves the gap until they are adjacent, so each commit is probed at most once and the probes total at
/// most 2 + ceil(log2(candidates.len())). When the results are not monotonic (a later commit passes again) the
/// answer is still a failing commit whose predecessor (or `last_pass`) passed, though not always the oldest
/// such one. A probe's Err is returned at once, with no further probe.
pub fn search_first_failing<E>(
    last_pass: &Oid,
    candidates: &[Oid],
    mut passes: impl FnMut(&Oid) -> Result<bool, E>,
) -> Result<BisectVerdict, E> {
    let Some(failed) = candidates.last() else {
        return Ok(BisectVerdict::NotFound(match passes(last_pass)? {
            true => BisectNotFound::FailedPasses,
            false => BisectNotFound::LastPassFails,
        }));
    };
    if passes(failed)? {
        return Ok(BisectVerdict::NotFound(BisectNotFound::FailedPasses));
    }
    if !passes(last_pass)? {
        return Ok(BisectVerdict::NotFound(BisectNotFound::LastPassFails));
    }
    // Positions: 0 is last_pass, i + 1 is candidates[i]. `lo` always passes and `hi` always fails.
    let (mut lo, mut hi) = (0, candidates.len());
    while hi - lo > 1 {
        let mid = lo + (hi - lo) / 2;
        if passes(&candidates[mid - 1])? {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    Ok(BisectVerdict::FirstFailing(candidates[hi - 1].clone()))
}
