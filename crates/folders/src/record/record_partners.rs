//! A record's "partners" value back to the partners.
//! No unit test: decode_record_and_partners_tests covers it (partner text, zone name, object id refusals).

use gitio::Oid;

use crate::plan::check_name;

use super::{FolderKind, RecordJsonValue, RequestPartner};

/// The partners in `slot`, the "partners" key's value or None when absent, on a record of kind `kind`. Only a
/// request record may have the key, and when present it holds at least one "<zone>:<tip>" string: a valid zone
/// name, then a git object id after the first ':'. Absent means no partners.
pub(crate) fn record_partners(
    slot: Option<RecordJsonValue>,
    kind: FolderKind,
) -> Result<Vec<RequestPartner>, String> {
    let Some(value) = slot else {
        return Ok(Vec::new());
    };
    match kind {
        FolderKind::Zone => return Err("\"partners\" key on a zone record".to_string()),
        FolderKind::Attempt(_) => return Err("\"partners\" key on an attempt record".to_string()),
        FolderKind::Request => {}
    }
    let RecordJsonValue::Strings(items) = value else {
        return Err("partners is not an array of strings".to_string());
    };
    if items.is_empty() {
        return Err("partners is empty; a record with no partners has no \"partners\" key".to_string());
    }
    items.iter().map(|item| partner(item)).collect()
}

fn partner(item: &str) -> Result<RequestPartner, String> {
    let refused = || format!("partner {item:?} is not a zone name, ':' and a git object id");
    let (zone, tip) = item.split_once(':').ok_or_else(refused)?;
    check_name("zone", zone).map_err(|_| refused())?;
    Ok(RequestPartner {
        zone: zone.to_string(),
        tip: Oid::parse(tip).map_err(|_| refused())?,
    })
}
