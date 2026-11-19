//! observe_item: What stands at a recorded item's path now, read without following a link.
//! No sibling test: it touches the disk. The install integration tests in install/integration/ cover it: uninstall.rs
//! (uninstall_keeps_and_names_edited_skill_but_removes_binary_and_link sees a changed file, a file and a link).

use std::fs;
use std::io::ErrorKind;

use super::{content_hash, ItemState, RecordItem};

/// Gone, Link with its target as read, File with content_hash of its bytes, Dir, or Other (anything else, or a path
/// that could not be read).
pub fn observe_item(item: &RecordItem) -> ItemState {
    let (RecordItem::File { path, .. } | RecordItem::Link { path, .. } | RecordItem::Dir { path }) = item;
    match fs::symlink_metadata(path) {
        Err(error) if error.kind() == ErrorKind::NotFound => ItemState::Gone,
        Err(_) => ItemState::Other,
        Ok(meta) if meta.file_type().is_symlink() => {
            fs::read_link(path).map_or(ItemState::Other, |target| ItemState::Link { target })
        }
        Ok(meta) if meta.is_file() => {
            fs::read(path).map_or(ItemState::Other, |bytes| ItemState::File { hash: content_hash(&bytes) })
        }
        Ok(meta) if meta.is_dir() => ItemState::Dir,
        Ok(_) => ItemState::Other,
    }
}
