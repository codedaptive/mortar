//! Which kind of folder a record describes.
//! A bare type; encode_folder_record_tests and decode_folder_record_tests exercise each kind.

/// `Zone`: a lane's standing folder. `Request`: one request's folder, cut inside a zone. `Attempt(k)`: attempt
/// folder k of a request, numbered from 1 (S-69).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FolderKind {
    Zone,
    Request,
    Attempt(u32),
}
