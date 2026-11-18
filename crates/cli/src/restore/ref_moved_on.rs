//! ref_moved_on: whether a repeated `mr restore` leaves a folder alone because its ref has moved on since the
//! done record it repeats (S-113(3)).
//! Its ZUT is ref_moved_on_tests; the RIT crates/cli/tests/cli_rit/run_restore_repeat_tests.rs covers it end to end
//! (a repeat after a later restore leaves the folder and its ref).

use gitio::Oid;

/// True exactly when `repeated` and the ref's `current` value is not `new`, the done record's new value for it; a
/// missing ref counts as moved on. A repeat finishes the record's move only while the ref still holds its new value
/// (S-113(3)): any other value is a later mover's, which the repeat leaves. A first attempt (`repeated` false) is
/// never moved on, whatever the ref holds.
pub fn ref_moved_on(repeated: bool, current: Option<&Oid>, new: &Oid) -> bool {
    repeated && current != Some(new)
}
