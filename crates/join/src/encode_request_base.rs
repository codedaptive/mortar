//! encode_request_base: a RequestBase as the text a join op records under BASE_BEFORE or BASE_AFTER.
//! encode_request_base_tests covers a base with no partners and one with two, a zone name escaped;
//! decode_request_base_tests reads this encoder's output back.

use crate::request_base::RequestBase;
use crate::write_partners::write_partners;

/// Exactly `{"base":"<hex>","partners":[{"zone":<string>,"tip":"<hex>"},...]}` with no whitespace, the partners in
/// the base's order and each zone name written by write_json_string.
pub(crate) fn encode_request_base(base: &RequestBase) -> String {
    // Ids are hex, so they need no escaping.
    let mut out = format!("{{\"base\":\"{}\",\"partners\":", base.base);
    write_partners(&mut out, &base.partners);
    out.push('}');
    out
}
