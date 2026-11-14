//! Which kind of folder a record describes.
//! A bare type; encode_folder_record_tests and decode_folder_record_tests exercise both kinds.

/// `Zone`: a lane's standing folder. `Request`: one request's folder, cut inside a zone.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FolderKind {
    Zone,
    Request,
}
