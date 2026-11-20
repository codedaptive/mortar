//! Whether last_pass is on failed's first-parent line, read from that line's own commits.

use gitio::Oid;

/// True when `base` is on `tip`'s first-parent line, so bisecting from `base` to `tip` names landings.
///
/// `line` is first_parent_list(base, tip): `tip`'s first-parent commits that `base` cannot reach, oldest first.
/// `oldest_first_parent` is `line[0]`'s first parent, None for a root commit or an empty line. `base` is on the
/// line exactly when the walk stops at it: either the line is empty and `base` is `tip`, or the oldest commit's
/// first parent is `base`. A `base` reached only as a merge's second parent, a `base` that is no ancestor (the
/// walk ends at a root), or a `tip` that `base` already reaches (an empty line with `base` != `tip`) is off it.
pub(crate) fn on_first_parent_line(base: &Oid, tip: &Oid, line: &[Oid], oldest_first_parent: Option<&Oid>) -> bool {
    match line.is_empty() {
        true => base == tip,
        false => oldest_first_parent == Some(base),
    }
}
