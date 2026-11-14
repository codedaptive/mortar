//! What mortar records about a folder it made (S-18), kept in that folder's git dir.
//! A bare type; encode_folder_record_tests and decode_folder_record_tests exercise it.

use gitio::Oid;

use super::FolderKind;

/// A folder's record. `request` names the request for a Request or an Attempt folder and is None for a Zone
/// folder; decode_folder_record refuses any other pairing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FolderRecord {
    pub kind: FolderKind,
    pub zone: String,
    pub request: Option<String>,
    /// The commit the folder was cut from.
    pub base: Oid,
    pub work_order_paths: Vec<String>,
    pub needed_paths: Vec<String>,
    /// The branch the folder was cut from; None for a zone folder.
    pub parent: Option<String>,
}
