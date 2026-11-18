//! encode_record: Install's record as the one JSON line written to install.json.

use std::path::{Path, PathBuf};

use oplog::write_json_string;

use super::RecordItem;

/// `{"items":[...]}` and a newline; Err the first path that is not UTF-8.
pub fn encode_record(items: &[RecordItem]) -> Result<String, PathBuf> {
    let mut out = String::from("{\"items\":[");
    for (index, item) in items.iter().enumerate() {
        if index > 0 {
            out.push(',');
        }
        match item {
            RecordItem::File { path, hash } => {
                out.push_str("{\"kind\":\"file\"");
                push_member(&mut out, "path", utf8(path)?);
                push_member(&mut out, "hash", hash);
            }
            RecordItem::Link { path, target } => {
                out.push_str("{\"kind\":\"link\"");
                push_member(&mut out, "path", utf8(path)?);
                push_member(&mut out, "target", utf8(target)?);
            }
            RecordItem::Dir { path } => {
                out.push_str("{\"kind\":\"dir\"");
                push_member(&mut out, "path", utf8(path)?);
            }
        }
        out.push('}');
    }
    out.push_str("]}\n");
    Ok(out)
}

/// `,"key":"value"`, the value escaped as JSON.
fn push_member(out: &mut String, key: &str, value: &str) {
    out.push(',');
    write_json_string(out, key);
    out.push(':');
    write_json_string(out, value);
}

fn utf8(path: &Path) -> Result<&str, PathBuf> {
    path.to_str().ok_or_else(|| path.to_path_buf())
}
