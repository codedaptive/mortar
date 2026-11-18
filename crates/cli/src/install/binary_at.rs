//! What stands at `~/.local/bin/mortar` before install places the binary (spec 07 item 1).
//! A bare type (S-7); plan_placement_tests exercises it.

/// Absent: nothing there. Same: this binary (its canonical path, or the same bytes). Mortar: another Mortar build
/// (S-147's test passes). Foreign: anything else, which install refuses to replace.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryAt {
    Absent,
    Same,
    Mortar,
    Foreign,
}
