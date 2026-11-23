//! make_partner_merge: merges a zone tip with its partners' tips into one commit, the base a request is made on.
//! No unit test: it only composes git calls (check_onto, is_ancestor, commit_info, merge_tree, commit_tree).
//! tests/partner_base covers it in a throwaway repository: partner_merge_is_deterministic_and_omits_ancestors (two
//! runs give the same commit, parents in order, the zone tip's author as author and committer, the merged tree,
//! partners already in the zone dropped, none left answering the zone tip) and
//! partner_merge_conflict_refuses_without_moving_refs_or_writing_state.

use crate::check_onto::check_onto;
use crate::partner::Partner;
use crate::partner_merge::PartnerMerge;
use crate::request_base::RequestBase;
use crate::JoinError;
use gitio::{Git, MergeTreeOutcome, Oid};
use std::iter::once;

/// Checks `zone_tip` exists first (OntoMissing), since every merge reads its merge attributes from it. Drops each
/// partner whose tip is already in the zone (an ancestor of `zone_tip`, or equal); with none left it answers
/// `zone_tip` itself and an empty partner list. Otherwise merges the kept partners into the zone tip one by one in
/// the order given, each merge chained on a commit of the one before; the first conflict answers Conflicted naming
/// that partner. The last merged tree is committed with parents `zone_tip` then the kept tips in order. Every commit
/// takes the zone tip's author as author and committer, and a message built from the ids and zone names alone, so
/// the same inputs always give the same commit. Writes objects; moves no ref and writes no state.
pub fn make_partner_merge(git: &Git, zone_tip: &Oid, partners: &[Partner]) -> Result<PartnerMerge, JoinError> {
    check_onto(git, zone_tip)?;
    let mut kept = Vec::new();
    for partner in partners {
        if !git.is_ancestor(&partner.tip, zone_tip)? {
            kept.push(partner.clone());
        }
    }
    let author = git.commit_info(zone_tip)?.author;
    let message = merge_message(zone_tip, &kept);
    let mut running = zone_tip.clone();
    for (index, partner) in kept.iter().enumerate() {
        let tree = match git.merge_tree(zone_tip, None, &running, &partner.tip)? {
            MergeTreeOutcome::Clean(tree) => tree,
            MergeTreeOutcome::Conflicted { files, .. } => {
                return Ok(PartnerMerge::Conflicted { zone: partner.zone.clone(), files })
            }
        };
        let parents = if index + 1 < kept.len() {
            vec![running, partner.tip.clone()]
        } else {
            once(zone_tip.clone()).chain(kept.iter().map(|kept| kept.tip.clone())).collect()
        };
        running = git.commit_tree(&tree, &parents, &message, Some(&author), Some(&author))?;
    }
    Ok(PartnerMerge::Merged(RequestBase { base: running, partners: kept }))
}

/// `mortar partner merge onto <zone_tip>`, a blank line, then `partner <zone> <tip>` for each kept partner in order.
fn merge_message(zone_tip: &Oid, kept: &[Partner]) -> String {
    let lines: String = kept.iter().map(|partner| format!("partner {} {}\n", partner.zone, partner.tip)).collect();
    format!("mortar partner merge onto {zone_tip}\n\n{lines}")
}
