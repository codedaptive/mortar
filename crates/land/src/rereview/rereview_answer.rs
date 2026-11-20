//! What a re-review answers: its text and whether the patch is the same.
//! A bare type, so no sibling test; crates/land/tests/rereview_zit.rs covers it through answer_rereview, and
//! same_patch_tests.rs covers the rule that sets `same_patch`.

/// The re-review of `new` against `reviewed`, two versions of one squash.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RereviewAnswer {
    /// Exactly what rereview returns: the "nothing changed" line, or git's range-diff.
    pub text: String,
    /// True when `new` carries the same patch as `reviewed`: the same tree and parents, or both commits' patch IDs,
    /// each against its own first parent, present and equal. A fact about the two commits only: it says nothing
    /// about whether the review still holds on the new base.
    pub same_patch: bool,
}
