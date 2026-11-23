//! request_base_of: a folder record's RecordedBase as the join crate's RequestBase, for a join or a resolve.

use ::join::{Partner, RequestBase};
use folders::RecordedBase;

/// The same base and the same partners, field for field, in the same order.
pub(crate) fn request_base_of(base: &RecordedBase) -> RequestBase {
    RequestBase {
        base: base.base.clone(),
        partners: base
            .partners
            .iter()
            .map(|partner| Partner { zone: partner.zone.clone(), tip: partner.tip.clone() })
            .collect(),
    }
}
