//! recorded_base_of: a join RequestBase as the folders crate's RecordedBase, for writing it to a folder record.

use ::join::RequestBase;
use folders::{RecordedBase, RequestPartner};

/// The same base and the same partners, field for field, in the same order.
pub(crate) fn recorded_base_of(base: &RequestBase) -> RecordedBase {
    RecordedBase {
        base: base.base.clone(),
        partners: base
            .partners
            .iter()
            .map(|partner| RequestPartner { zone: partner.zone.clone(), tip: partner.tip.clone() })
            .collect(),
    }
}
