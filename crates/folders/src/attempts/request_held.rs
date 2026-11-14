//! Whether keep may move the request branch to the kept attempt's tip without dropping a commit.

use gitio::Oid;

/// True when `merge_base`, the merge base of `request_tip` and the kept tip, is `request_tip` itself: the kept tip
/// descends from the request tip, so moving the request branch there drops no commit. False otherwise, also when
/// the two share no merge base (None); keep then refuses RequestMovedOn.
pub fn request_held(request_tip: &Oid, merge_base: Option<&Oid>) -> bool {
    merge_base == Some(request_tip)
}
