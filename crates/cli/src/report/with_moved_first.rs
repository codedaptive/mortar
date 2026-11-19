//! with_moved_first: a report's lines with moved_before's line ahead of them. Every report that names what earlier
//! steps moved puts that line before any claim about what did not move; join, resolve and save all build it here,
//! in report/, the lowest level covering the three.

/// `moved`, when given, as the first line, then `lines` in their order; `lines` alone when `moved` is None.
pub(crate) fn with_moved_first(moved: Option<&str>, lines: Vec<String>) -> Vec<String> {
    moved.map(str::to_string).into_iter().chain(lines).collect()
}
