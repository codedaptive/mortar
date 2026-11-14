//! How a folder refusal reads: each names the path, branch or error it carries, if it carries one.

use std::fmt;

use super::FolderError;
use crate::record::{FolderKind, FolderRecord};

impl fmt::Display for FolderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FolderError::Plan(e) => write!(f, "{e}"),
            FolderError::Record { path, error } => write!(f, "{}: {error}", path.display()),
            FolderError::Git(e) => write!(f, "{e}"),
            FolderError::Oplog(e) => write!(f, "{e}"),
            FolderError::Lock(e) => write!(f, "folders lock: {e}"),
            FolderError::BelowFloor(e) => write!(f, "folder not made: {e}"),
            FolderError::DiskProbe { path, reason } => {
                write!(f, "cannot read free disk at {}: {reason}", path.display())
            }
            FolderError::DifferentFolder { path, existing } => write!(
                f,
                "{} is already {}; remove it or choose another path",
                path.display(),
                folder_text(existing)
            ),
            FolderError::NotAFolder { path, reason } => {
                write!(f, "{} is not a mortar folder: {reason}", path.display())
            }
            FolderError::BranchCheckedOut { branch, path } => {
                write!(f, "branch {branch} is already checked out at {}", path.display())
            }
            FolderError::BranchExists { branch } => write!(
                f,
                "branch {branch} already exists at another commit or was not made by mortar at this one; \
                 the folder was not made"
            ),
            FolderError::ZoneBranchMissing { branch } => write!(f, "zone branch {branch} does not exist"),
            FolderError::Io { path, source } => write!(f, "{}: {source}", path.display()),
        }
    }
}

/// "the zone folder for zone Z", "the request folder for zone Z, request R", or, for a request record with no
/// request, "a request folder for zone Z".
fn folder_text(record: &FolderRecord) -> String {
    match (record.kind, &record.request) {
        (FolderKind::Request, Some(request)) => {
            format!("the request folder for zone {}, request {request}", record.zone)
        }
        (FolderKind::Request, None) => format!("a request folder for zone {}", record.zone),
        (FolderKind::Zone, _) => format!("the zone folder for zone {}", record.zone),
    }
}
