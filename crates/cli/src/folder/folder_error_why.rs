//! Why a `mortar folder` was refused, as facts read from the error's fields. folders' own Display text is never
//! used: it carries next steps that are not cli commands (S-112), and BelowFloor's would print what a sweep frees,
//! which mortar folder does not measure.

use std::path::PathBuf;

use folders::FolderError;
use gitio::Oid;

use super::folder_kind_text;

/// The lines for every variant folder_refusal answers for; none for Lock, Git, Oplog and DiskProbe, which are
/// reported as every cli verb reports them. Paths, branches, refs, k, the cap and refused files come from the
/// fields; Plan gives its reason, Record its path and the record's reason, Io its path and the io error's text. No
/// line holds a next step.
pub fn folder_error_why(error: &FolderError) -> Vec<String> {
    match error {
        FolderError::Lock(_) | FolderError::Git(_) | FolderError::Oplog(_) | FolderError::DiskProbe { .. } => {
            Vec::new()
        }
        FolderError::Plan(plan) => vec![format!("folder plan: {}", plan.reason)],
        FolderError::Record { path, error } => {
            vec![format!("{}: the folder record is not one Mortar writes: {}", path.display(), error.reason)]
        }
        FolderError::Io { path, source } => vec![format!("{}: {source}", path.display())],
        FolderError::BelowFloor(below) => vec![format!(
            "free disk {} bytes is under the floor of {} bytes; nothing was made",
            below.free, below.floor
        )],
        FolderError::DifferentFolder { path, existing } => {
            vec![format!("{} is already {}, not the folder asked for", path.display(), folder_kind_text(existing))]
        }
        FolderError::NotAFolder { path, reason } => {
            vec![format!("{} is in the way and is not a Mortar folder: {reason}", path.display())]
        }
        FolderError::BranchCheckedOut { branch, path } => {
            vec![format!("branch {branch} is checked out in the folder at {}", path.display())]
        }
        FolderError::BranchExists { branch } => {
            vec![format!("branch {branch} already exists at another commit, or Mortar did not make it at this one")]
        }
        FolderError::ZoneBranchMissing { branch } => vec![format!("zone branch {branch} does not exist")],
        FolderError::RequestBranchMissing { branch } => vec![format!("request branch {branch} does not exist")],
        FolderError::AttemptMissing { branch } => vec![format!("attempt branch {branch} does not exist")],
        FolderError::AttemptsUnsaved { paths } => {
            each(paths, "the work of", "is not saved", "nothing changed")
        }
        FolderError::AttemptsRemain { path, attempts } => vec![
            format!("attempts of the request at {} remain: {}", path.display(), attempts.join(", ")),
            "nothing changed".to_string(),
        ],
        FolderError::OrphanAttempt { paths } => {
            each(paths, "the attempt folder", "has lost its try branch", "nothing changed")
        }
        FolderError::OrphanWithoutSnapshot { paths } => each(
            paths,
            "the attempt folder",
            "has lost its try branch, and Mortar holds no copy of it",
            "nothing changed",
        ),
        FolderError::RequestMovedOn { branch, attempt } => vec![
            format!("request branch {branch} has commits attempt branch {attempt} does not hold"),
            "nothing changed".to_string(),
        ],
        FolderError::AttemptFolder { path, k } => vec![
            format!("{} is attempt folder {k}; keep and drop remove attempt folders", path.display()),
            "nothing changed".to_string(),
        ],
        FolderError::NotSaved { path } => vec![format!("{} holds work that is not saved", path.display())],
        FolderError::MoveRefused { path, paths } => vec![
            format!("{} was not moved: local changes are in the way at {}", path.display(), joined(paths)),
            "nothing changed".to_string(),
        ],
        FolderError::BranchElsewhere { branch, at, old, new } => vec![format!(
            "branch {branch} is at {}, neither the move's old tip {} nor its new tip {}",
            at_text(at.as_ref()),
            old.as_str(),
            new.as_str()
        )],
        FolderError::NotRemovedUnsaved { path, refused, cap } if !refused.is_empty() => vec![format!(
            "{} holds files the snapshot size cap{} refused: {}; they are in no snapshot",
            path.display(),
            cap.map_or(String::new(), |bytes| format!(" of {bytes} bytes")),
            joined(refused)
        )],
        FolderError::NotRemovedUnsaved { path, .. } => {
            vec![format!("{} holds work that is not saved", path.display())]
        }
        FolderError::NotJoined { path, branch, zone_branch } => {
            vec![format!("branch {branch} of {} is not contained in {zone_branch}", path.display())]
        }
        FolderError::ZoneFolder { path, zone } => vec![
            format!("{} is the zone folder of zone {zone}", path.display()),
            "nothing changed".to_string(),
        ],
        FolderError::RefMoved { refname, at } => vec![format!(
            "ref {refname} moved while it was being deleted; it is now at {}; nothing was deleted",
            at_text(at.as_ref())
        )],
        FolderError::AttemptRefsLeft { moved, at, removed } => {
            let mut lines = vec![format!(
                "ref {moved} moved before the op ran; it is now at {}; every attempt branch is kept",
                at_text(at.as_ref())
            )];
            lines.extend(removed.iter().map(|path| format!("the attempt folder {} was removed", path.display())));
            lines
        }
        FolderError::RemovedRefsLeft { path, branch, moved, at } => vec![
            format!("{} was removed", path.display()),
            format!(
                "ref {moved} moved (now at {}) before branch {branch} and its snapshot ref were deleted; both are left",
                at_text(at.as_ref())
            ),
        ],
        FolderError::ExtraRefRefused { refname, reason } => vec![format!("extra ref {refname} {reason}")],
    }
}

/// One line per path, "<before> <path> <after>", then `last`.
fn each(paths: &[PathBuf], before: &str, after: &str, last: &str) -> Vec<String> {
    let mut lines: Vec<String> = paths.iter().map(|path| format!("{before} {} {after}", path.display())).collect();
    lines.push(last.to_string());
    lines
}

fn joined(paths: &[PathBuf]) -> String {
    paths.iter().map(|path| path.display().to_string()).collect::<Vec<_>>().join(", ")
}

fn at_text(at: Option<&Oid>) -> &str {
    at.map_or("no commit", Oid::as_str)
}
