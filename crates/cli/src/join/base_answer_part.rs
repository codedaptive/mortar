//! The lines, fields, exit, outcome and next step a join on a request's recorded base gives its report.
//! Shell API: join_report calls it.

use ::join::{BaseJoinAnswer, JoinAnswer};

use super::{answer_part, AnswerPart};
use crate::folder::partners_json;
use crate::report::{ExitCode, OutValue};
use crate::request_base::recorded_base_of;

/// Plain is answer_part's. ZoneJoined is exit 0 "joined", next "wait for the zone's tests": one line naming both
/// moves, as zone and partners stood where the request's base had them and the merge's tree already ran its tests
/// in the folder (S-231). Rebased with partners is exit 0 "rebased", next "run the unit tests, then mr join": the
/// request now stands on a new merge of the zone with its partners, whose tree no test has run on. Rebased with no
/// partners (every partner already in the zone, so the base is the zone tip) is answer_part's plain Rebased.
/// `moved` (moved_before's line) ends the first line of ZoneJoined and Rebased. PartnersConflicted
/// (partners_conflicted_report's) and Plain(FilesWritten), which only resolve gives, are None.
pub(crate) fn base_answer_part(answer: &BaseJoinAnswer, request_ref: &str, moved: Option<&str>) -> Option<AnswerPart> {
    let then_moved = moved.map_or(String::new(), |moved| format!("; {moved}"));
    Some(match answer {
        BaseJoinAnswer::Plain(plain) => return answer_part(plain, request_ref, moved),
        BaseJoinAnswer::Rebased { from, to, base, .. } if base.partners.is_empty() => {
            let plain = JoinAnswer::Rebased {
                request_ref: request_ref.to_string(),
                from: from.clone(),
                to: to.clone(),
                folder_tree: None,
            };
            return answer_part(&plain, request_ref, moved);
        }
        BaseJoinAnswer::ZoneJoined { zone_ref, from, request_from, to, .. } => AnswerPart {
            lines: vec![format!(
                "joined: {zone_ref} moved from {from} to {to}, taking only the request's own commits; {request_ref} \
                 moved from {request_from} to {to} and the folder moved with it{then_moved}"
            )],
            fields: vec![
                text("zone_ref", zone_ref),
                text("from", from),
                text("to", to),
                text("request_from", request_from),
            ],
            exit: ExitCode::Done,
            outcome: "joined",
            next: "wait for the zone's tests",
        },
        BaseJoinAnswer::Rebased { from, to, base, .. } => AnswerPart {
            lines: vec![
                format!(
                    "rebased: {request_ref} moved from {from} to {to}, replayed onto a new merge {} of the zone with \
                     its partners; the folder moved with it{then_moved}",
                    base.base
                ),
                "the unit tests must run again on the new merge".to_string(),
            ],
            fields: vec![
                text("from", from),
                text("to", to),
                text("base", &base.base),
                ("partners".to_string(), OutValue::Raw(partners_json(&recorded_base_of(base).partners))),
            ],
            exit: ExitCode::Done,
            outcome: "rebased",
            next: "run the unit tests, then mr join",
        },
        BaseJoinAnswer::PartnersConflicted { .. } => return None,
    })
}

fn text(name: &str, value: impl ToString) -> (String, OutValue) {
    (name.to_string(), OutValue::Text(value.to_string()))
}
