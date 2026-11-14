//! Which leftover request branches have joined their zone, chosen from git's merge bases as data.

use gitio::Oid;

use super::Leftover;

/// Each candidate is a leftover branch with git's merge base of its tip and its zone_tip (None when they share no
/// history). Keeps, in input order, those whose merge base is Some(its tip): the branch is contained in its zone,
/// so joined (S-52 (1)); drops the rest. sweep_refs and the disk report's ref_sweep_bytes both choose joined
/// leftovers with it.
pub fn joined_leftovers(candidates: Vec<(Leftover, Option<Oid>)>) -> Vec<Leftover> {
    candidates
        .into_iter()
        .filter(|(leftover, base)| base.as_ref() == Some(&leftover.tip))
        .map(|(leftover, _)| leftover)
        .collect()
}
