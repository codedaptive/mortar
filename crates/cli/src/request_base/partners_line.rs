//! partners_line: a request folder's partners as one line of text for a report.

use folders::RequestPartner;

/// "partners: <zone> at <tip>, <zone> at <tip>" with full hex tips, in the order given; "partners: none" when
/// there are none.
pub(crate) fn partners_line(partners: &[RequestPartner]) -> String {
    if partners.is_empty() {
        return "partners: none".to_string();
    }
    let listed: Vec<String> = partners.iter().map(|partner| format!("{} at {}", partner.zone, partner.tip)).collect();
    format!("partners: {}", listed.join(", "))
}
