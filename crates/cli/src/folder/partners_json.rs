//! partners_json: a request folder's partners as JSON objects, for the `partners` member of a report.

use folders::RequestPartner;
use oplog::write_json_string;

/// One encoded object per partner, in the order given, for `OutValue::Raw`: `{"zone":..,"tip":".."}` with the
/// tip's full hex; no partners gives an empty Vec.
pub(crate) fn partners_json(partners: &[RequestPartner]) -> Vec<String> {
    partners
        .iter()
        .map(|partner| {
            let mut out = String::from("{\"zone\":");
            write_json_string(&mut out, &partner.zone);
            out.push_str(",\"tip\":\"");
            out.push_str(partner.tip.as_str());
            out.push_str("\"}");
            out
        })
        .collect()
}
