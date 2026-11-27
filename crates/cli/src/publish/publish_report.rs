//! The report of an `mr publish` that land's publish_with_push answered, with `pushed` on every answer (land's: true
//! only when this publish's push intent closed done, so the published commit is on the remote by this publish's push
//! or an earlier one of this landing's).

use std::path::Path;

use gitio::Oid;
use land::{LandRefs, PublishAnswer};

use crate::integration_folder::integration_folder_line;
use crate::report::{ExitCode, OutValue, Report};

const VERB: &str = "publish";

/// `pushed` is land's PublishOutcome's; `push` is false for `mr publish --no-push`. Every answer carries the field
/// pushed (a boolean).
/// `zones` are the published batch's zones in landing order, read before land removed the state (empty when it was
/// already gone).
/// - Published: exit 0 "published", the landing state and the squash refs removed (removed_line); field origin. When
///   pushed, its line says the remote's and the local origin branch are at `origin`; otherwise it names only the local
///   origin branch and says this publish pushed nothing to the remote. Next names `mr after-publish` for each zone
///   (after_publish_next).
/// - OriginMoved: exit 1 "origin-moved", nothing published; the local origin branch and integration are on `found`,
///   the squash kept, and `folder` (the integration folder moved onto `found`, if any) is named. Push on: someone
///   else moved the remote's origin branch to `found`, owned by whoever pushed it. Push off: someone else moved the
///   local origin branch to `found`, nothing was fetched, owned by whoever moved it. Next lands the squash again onto
///   `found` with `mr land <commit>`, then publishes with the same setting. Fields found, integration (also
///   `found`) and folder (its path, or null).
/// - NothingToPublish: exit 0 "nothing-to-publish", no landing waiting.
pub fn publish_report(
    answer: &PublishAnswer,
    pushed: bool,
    push: bool,
    refs: &LandRefs,
    zones: &[String],
    folder: Option<&Path>,
) -> Report {
    let remote = &refs.remote;
    let origin_branch = &refs.origin_branch;
    let pushed_field = ("pushed".to_string(), OutValue::Bool(pushed));
    match answer {
        PublishAnswer::Published { origin } => Report {
            verb: VERB.to_string(),
            exit: ExitCode::Done,
            outcome: "published".to_string(),
            lines: vec![
                if pushed {
                    format!("published {origin}: {remote}'s {origin_branch} and the local {origin_branch} are at it")
                } else {
                    format!(
                        "published {origin}: the local {origin_branch} is at it; this publish pushed nothing to \
                         {remote}"
                    )
                },
                removed_line(zones),
            ],
            owner: None,
            next: after_publish_next(zones),
            fields: vec![("origin".to_string(), OutValue::Text(origin.to_string())), pushed_field],
        },
        PublishAnswer::OriginMoved { found } => {
            let mut report = origin_moved(found, push, refs, folder);
            report.fields.push(pushed_field);
            report
        }
        PublishAnswer::NothingToPublish => Report {
            verb: VERB.to_string(),
            exit: ExitCode::Done,
            outcome: "nothing-to-publish".to_string(),
            lines: vec!["no landing is waiting to be published".to_string()],
            owner: None,
            next: "nothing to do".to_string(),
            fields: vec![pushed_field],
        },
    }
}

/// Published's second line. A batch of one (or a state already gone) keeps the single-zone wording; a batch of more
/// names its zones.
fn removed_line(zones: &[String]) -> String {
    if zones.len() > 1 {
        format!("the landing state and the squash refs of zones {} are removed", zones.join(", "))
    } else {
        "the landing state and the zone's squash ref are removed".to_string()
    }
}

/// Published's next: `mr after-publish` for each zone in landing order; with no zones read, the general form.
fn after_publish_next(zones: &[String]) -> String {
    if zones.is_empty() {
        return "mr after-publish <zone> --actor <name> for each zone this landing published".to_string();
    }
    zones.iter().map(|zone| format!("mr after-publish {zone} --actor <name>")).collect::<Vec<_>>().join(", then ")
}

/// OriginMoved's report without the pushed field: push on names the remote's origin branch, push off the local one.
fn origin_moved(found: &Oid, push: bool, refs: &LandRefs, folder: Option<&Path>) -> Report {
    let remote = &refs.remote;
    let origin_branch = &refs.origin_branch;
    let (moved, owner, repeat) = if push {
        (
            format!("someone else moved {remote}'s {origin_branch} to {found}; nothing was published"),
            format!("whoever pushed {found} to {remote}"),
            "mr publish --actor <name>",
        )
    } else {
        (
            format!(
                "someone else moved the local {origin_branch} to {found}; nothing was published and nothing was \
                 fetched"
            ),
            format!("whoever moved the local {origin_branch} to {found}"),
            "mr publish --no-push --actor <name>",
        )
    };
    Report {
        verb: VERB.to_string(),
        exit: ExitCode::Refused,
        outcome: "origin-moved".to_string(),
        lines: vec![
            moved,
            format!("the local {origin_branch} and integration are on {found}; the squash and its ref are kept"),
            integration_folder_line(folder, found),
        ],
        owner: Some(owner),
        next: format!("land the change's squash again onto {found}: mr land <commit> --actor <name>, then {repeat}"),
        fields: vec![
            ("found".to_string(), OutValue::Text(found.to_string())),
            ("integration".to_string(), OutValue::Text(found.to_string())),
            ("folder".to_string(), folder.map_or(OutValue::Null, |path| OutValue::Text(path.display().to_string()))),
        ],
    }
}
