//! How a folder refusal reads: each names the path, branch or error it carries, if it carries one.

use std::fmt;
use std::path::{Path, PathBuf};

use gitio::Oid;

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
            FolderError::NotSaved { path } => write!(
                f,
                "{} holds work that is not saved; the folder was not moved; next step: mortar save",
                path.display()
            ),
            FolderError::MoveRefused { path, paths } => write!(
                f,
                "{} was not moved: local changes are in the way at {}; nothing changed",
                path.display(),
                paths.iter().map(|p| p.display().to_string()).collect::<Vec<_>>().join(", ")
            ),
            FolderError::BranchElsewhere { branch, at, old, new } => write!(
                f,
                "branch {branch} is at {}, neither the move's old tip {} nor its new tip {}",
                at.as_ref().map_or("no commit", Oid::as_str),
                old.as_str(),
                new.as_str()
            ),
            FolderError::NotRemovedUnsaved { path, refused, cap } => not_removed_unsaved(f, path, refused, *cap),
            FolderError::NotJoined {
                path,
                branch,
                zone_branch,
            } => write!(
                f,
                "{} was not removed: branch {branch} is not contained in {zone_branch}; next step: mortar join",
                path.display()
            ),
            FolderError::ZoneFolder { path, zone } => write!(
                f,
                "{} is the zone folder for zone {zone}; remove takes request folders only; nothing changed",
                path.display()
            ),
            FolderError::RefMoved { refname, at } => write!(
                f,
                "ref {refname} moved while it was being deleted (now at {}); nothing was deleted; run again",
                at.as_ref().map_or("no commit", Oid::as_str)
            ),
            FolderError::RemovedRefsLeft { path, branch, moved, at } => write!(
                f,
                "{} was removed, but ref {moved} moved (now at {}) before branch {branch} and its snapshot ref were \
                 deleted; both are left as they are; the work they hold is in the zone; next step: mortar sweep",
                path.display(),
                at.as_ref().map_or("no commit", Oid::as_str)
            ),
            FolderError::ExtraRefRefused { refname, reason } => write!(
                f,
                "extra ref {refname} {reason}; sweep_refs deletes no ref under refs/heads/, refs/mortar/ or \
                 refs/remotes/, nor a protected one; nothing was swept"
            ),
            FolderError::Io { path, source } => write!(f, "{}: {source}", path.display()),
        }
    }
}

/// With no refused file, the unsaved text; with refused files, names them and the cap (in bytes, when known).
fn not_removed_unsaved(f: &mut fmt::Formatter<'_>, path: &Path, refused: &[PathBuf], cap: Option<u64>) -> fmt::Result {
    if refused.is_empty() {
        return write!(
            f,
            "{} holds work that is not saved; the folder was not removed; next step: mortar save",
            path.display()
        );
    }
    write!(
        f,
        "{} holds files the snapshot size cap{} refused: {}; they are in no snapshot, so the folder was not removed; \
         next step: move or delete them, then mortar save",
        path.display(),
        cap.map_or(String::new(), |n| format!(" ({n} bytes)")),
        refused.iter().map(|p| p.display().to_string()).collect::<Vec<_>>().join(", ")
    )
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
