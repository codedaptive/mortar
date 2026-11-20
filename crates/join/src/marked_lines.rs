//! marked_lines: the conflict-marker lines left in a file's bytes. marked_lines_tests covers two-way and diff3
//! conflicts, CRLF, longer markers, look-alike lines that are not markers, and bytes that are not UTF-8.

/// The 1-based numbers of the lines that are conflict markers. Lines are split on `\n`, one trailing `\r` ignored.
/// A line is a marker when it starts with 7 or more `<`, or 7 or more `>`, then a space or the end of the line.
/// Only between a `<` marker and the next `>` marker, a line starting with 7 or more `|` then a space or the end,
/// and a line of 7 or more `=` and nothing else, are markers too: an `=======` underline elsewhere is text.
/// 7 or more, not exactly 7, because git writes longer markers for a larger conflict-marker-size. Bytes, not
/// text, so a file that is not UTF-8 is checked too.
pub(crate) fn marked_lines(text: &[u8]) -> Vec<usize> {
    let mut marked = Vec::new();
    let mut in_conflict = false;
    for (index, raw) in text.split(|&byte| byte == b'\n').enumerate() {
        let line = raw.strip_suffix(b"\r").unwrap_or(raw);
        let marker = if starts_marker(line, b'<') {
            in_conflict = true;
            true
        } else if starts_marker(line, b'>') {
            in_conflict = false;
            true
        } else {
            in_conflict && (starts_marker(line, b'|') || is_separator(line))
        };
        if marker {
            marked.push(index + 1);
        }
    }
    marked
}

/// True when `line` starts with 7 or more `byte` followed by a space or the end of the line.
fn starts_marker(line: &[u8], byte: u8) -> bool {
    let run = line.iter().take_while(|&&b| b == byte).count();
    run >= 7 && matches!(line.get(run), None | Some(b' '))
}

/// True when `line` is 7 or more `=` and nothing else.
fn is_separator(line: &[u8]) -> bool {
    line.len() >= 7 && line.iter().all(|&b| b == b'=')
}
