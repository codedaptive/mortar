//! Whether a file's text holds git's conflict markers (step 1's checks 5 and 6, step 4's check 1).

/// True when any line starts with `<<<<<<<`, `>>>>>>>` or `|||||||` (the base marker git writes in diff3 style),
/// or is exactly `=======`.
pub fn holds_conflict_markers(text: &str) -> bool {
    text.lines().any(|line| {
        ["<<<<<<<", ">>>>>>>", "|||||||"].iter().any(|marker| line.starts_with(marker)) || line == "======="
    })
}
