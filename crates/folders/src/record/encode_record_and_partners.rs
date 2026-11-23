//! A folder record and its partners as one line of JSON.

use oplog::write_json_string;

use super::{encode_folder_record, FolderRecord, RequestPartner};

/// encode_folder_record's line for `record`, byte for byte when `partners` is empty; otherwise that line with the
/// key "partners" appended after "parent": an array of "<zone>:<tip>" strings in the order given.
pub(crate) fn encode_record_and_partners(record: &FolderRecord, partners: &[RequestPartner]) -> String {
    let mut out = encode_folder_record(record);
    if partners.is_empty() {
        return out;
    }
    out.pop();
    out.push_str(",\"partners\":[");
    for (i, partner) in partners.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        write_json_string(&mut out, &format!("{}:{}", partner.zone, partner.tip.as_str()));
    }
    out.push_str("]}");
    out
}
