//! OpenRequest: one open request of a zone, as overlap's caller hands it in. A declaration only; Nagatha's overlap
//! integration cases (listed in overlap.rs) are to exercise it.

use gitio::Oid;

/// An open request: its name, the commit it was cut from, and its tip, or its snapshot commit when its folder has
/// unsaved work. The caller picks which; join reads no folder records or snapshot refs.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OpenRequest {
    pub name: String,
    pub base: Oid,
    pub tip: Oid,
}
