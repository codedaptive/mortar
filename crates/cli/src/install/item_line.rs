//! item_line: One report item as a plain line.

use super::InstallItem;

/// The action, then the path when not empty, then ": " and the note when not empty.
pub fn item_line(item: &InstallItem) -> String {
    let mut line = item.action.clone();
    if !item.path.as_os_str().is_empty() {
        line.push(' ');
        line.push_str(&item.path.to_string_lossy());
    }
    if !item.note.is_empty() {
        line.push_str(": ");
        line.push_str(&item.note);
    }
    line
}
