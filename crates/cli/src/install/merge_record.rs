//! merge_record: An earlier record with a later install's items merged in (spec 07 item 7: a repeat keeps the earlier items).

use std::path::Path;

use super::RecordItem;

/// `old` in order, each item whose path is in `new` replaced in place by it; then the rest of `new` in order.
pub fn merge_record(old: &[RecordItem], new: &[RecordItem]) -> Vec<RecordItem> {
    let in_new = |path: &Path| new.iter().find(|item| item_path(item) == path);
    let mut merged: Vec<RecordItem> = old.iter().map(|item| in_new(item_path(item)).unwrap_or(item).clone()).collect();
    merged.extend(new.iter().filter(|item| !old.iter().any(|kept| item_path(kept) == item_path(item))).cloned());
    merged
}

fn item_path(item: &RecordItem) -> &Path {
    let (RecordItem::File { path, .. } | RecordItem::Link { path, .. } | RecordItem::Dir { path }) = item;
    path
}
