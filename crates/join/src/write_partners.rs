//! write_partners: a base's partners as the JSON array both mortar-join.json and a join op's base values hold.
//! No sibling test: encode_request_base_tests and encode_join_state_tests check its text through their encoders,
//! its two callers.

use crate::partner::Partner;
use oplog::write_json_string;
use std::fmt::Write;

/// Appends `[{"zone":<string>,"tip":"<hex>"},...]` to `out` with no whitespace, the partners in order and each zone
/// name written by write_json_string.
pub(crate) fn write_partners(out: &mut String, partners: &[Partner]) {
    out.push('[');
    for (i, partner) in partners.iter().enumerate() {
        out.push_str(if i > 0 { ",{\"zone\":" } else { "{\"zone\":" });
        write_json_string(out, &partner.zone);
        // Writing to a String cannot fail. Ids are hex, so they need no escaping.
        let _ = write!(out, ",\"tip\":\"{}\"}}", partner.tip);
    }
    out.push(']');
}
