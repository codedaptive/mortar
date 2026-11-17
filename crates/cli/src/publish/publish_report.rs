//! The report of a `mortar publish` that land's publish answered (SPEC-cli 06).

use std::path::Path;

use land::{LandRefs, PublishAnswer};

use crate::integration_folder::integration_folder_line;
use crate::report::{ExitCode, OutValue, Report};

const VERB: &str = "publish";

/// - Published: exit 0 "published", the remote's and the local origin branch at `origin`, the landing state and the
///   zone's squash ref removed; field origin.
/// - OriginMoved: exit 1 "origin-moved", someone else moved the remote's origin branch to `found`; the local origin
///   branch and integration are on it, the squash kept, and `folder` (the integration folder moved onto `found`,
///   if any) is named. Owned by whoever pushed `found`; next lands the squash again onto `found` with `mortar land
///   <commit>`, then publishes. Fields found, integration
///   (also `found`) and folder (its path, or null).
/// - NothingToPublish: exit 0 "nothing-to-publish", no landing waiting; no fields.
pub fn publish_report(answer: &PublishAnswer, refs: &LandRefs, folder: Option<&Path>) -> Report {
    let remote = &refs.remote;
    let origin_branch = &refs.origin_branch;
    match answer {
        PublishAnswer::Published { origin } => Report {
            verb: VERB.to_string(),
            exit: ExitCode::Done,
            outcome: "published".to_string(),
            lines: vec![
                format!("published {origin}: {remote}'s {origin_branch} and the local {origin_branch} are at it"),
                "the landing state and the zone's squash ref are removed".to_string(),
            ],
            owner: None,
            next: "nothing more for this landing; the next landing can start".to_string(),
            fields: vec![("origin".to_string(), OutValue::Text(origin.to_string()))],
        },
        PublishAnswer::OriginMoved { found } => Report {
            verb: VERB.to_string(),
            exit: ExitCode::Refused,
            outcome: "origin-moved".to_string(),
            lines: vec![
                format!("someone else moved {remote}'s {origin_branch} to {found}; nothing was published"),
                format!("the local {origin_branch} and integration are on {found}; the squash and its ref are kept"),
                integration_folder_line(folder, found),
            ],
            owner: Some(format!("whoever pushed {found} to {remote}")),
            next: format!(
                "land the change's squash again onto {found}: mortar land <commit> --actor <name>, then mortar \
                 publish --actor <name>"
            ),
            fields: vec![
                ("found".to_string(), OutValue::Text(found.to_string())),
                ("integration".to_string(), OutValue::Text(found.to_string())),
                (
                    "folder".to_string(),
                    folder.map_or(OutValue::Null, |path| OutValue::Text(path.display().to_string())),
                ),
            ],
        },
        PublishAnswer::NothingToPublish => Report {
            verb: VERB.to_string(),
            exit: ExitCode::Done,
            outcome: "nothing-to-publish".to_string(),
            lines: vec!["no landing is waiting to be published".to_string()],
            owner: None,
            next: "nothing to do".to_string(),
            fields: Vec::new(),
        },
    }
}
