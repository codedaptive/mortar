//! partner_moved: which partner zones moved since a request folder's base was made.

use folders::RequestPartner;
use gitio::Oid;

/// The zone names, in the order of `partners`, whose current tip (`current[i]` is the tip of refs/heads/zone/<zone>
/// of `partners[i]`) is None, the branch gone, or differs from the recorded tip.
pub fn partner_moved(partners: &[RequestPartner], current: &[Option<Oid>]) -> Vec<String> {
    partners
        .iter()
        .zip(current)
        .filter(|(partner, tip)| tip.as_ref() != Some(&partner.tip))
        .map(|(partner, _)| partner.zone.clone())
        .collect()
}
