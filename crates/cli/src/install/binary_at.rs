//! What stands at `~/.local/bin/mortar` before install places the binary.
//! A bare type; plan_placement_tests exercises it.

/// Absent: nothing there. Same: this binary (its canonical path, or the same bytes). Mortar: another Mortar build
/// (file_has_marker finds MORTAR_MARKER in it). Foreign: anything else, which install refuses to replace.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryAt {
    Absent,
    Same,
    Mortar,
    Foreign,
}
