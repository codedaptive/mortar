//! follow_answer: what `mr join` does after the join answers: rewrites the folder record and moves the folder after a
//! rebase, a joined partner request or a rebase onto a new merge, and reports a conflicting new merge.
//! No unit test: it runs git, reads the op log (read_pending_move), rewrites the folder record (rewrite_record_to) and
//! moves the folder; pending_base_tests, base_answer_part_tests and move_refusal_report_tests cover its decisions and
//! reports. The integration tests
//! crates/cli/tests/save_join_runs/{joins,partner_join_tests,plain_base_tests}.rs cover it through mr join, with the
//! mortar binary as a child process in a throwaway repository and a request folder. plain_base_tests.rs covers a plain
//! rebase, the record naming the zone tip with no partners, and a fast-forward, the record unchanged.
//! partner_join_tests.rs covers a request cut on a merge with a partner: unmoved, the zone takes only the request's
//! commits, the record naming the zone's tip before the join, with no partners; a moved partner rebases onto a new
//! merge the record names, next "run the unit tests, then mr join", and with every partner already in the zone it is
//! the plain rebase, the record naming the zone tip with no partners; a conflicting new merge is exit 1 naming the
//! partner and files, nothing moved.

use folders::{move_folder, RecordedBase};
use gitio::Oid;

use super::{move_refusal_report, partners_conflicted_report, read_pending_move, rewrite_record_to, RequestRun};
use crate::report::Report;
use crate::request_base::recorded_base_of;
use ::join::{BaseJoinAnswer, JoinAnswer};

const VERB: &str = "join";

/// Every answer that replays the request onto a zone tip rewrites the folder record to the request's new base before
/// the folder moves (S-239): a plain rebase to the base join's op recorded (read_pending_move's base for the move that
/// op left, the request ref standing at `to`), the zone tip replayed onto with no partners; a joined partner request
/// to the zone's tip before the join, with no partners; a rebase onto a new merge to that merge and its partners. The
/// record must not lag the branch: a record naming the old base would make the next join replay from a base the branch
/// no longer stands on, and the zone guard reads it as the join boundary.
/// A kill or a failed write between join's op and the rewrite leaves the record behind; join records the base in
/// its op, and the next finish_pending_move puts the record right. A plain rebase whose op recorded no base moves the
/// folder only. A conflicting new merge is partners_conflicted_report. Plain's other answers (a fast-forward, a
/// conflict) move nothing and leave the record.
pub(super) fn follow_answer(run: &RequestRun, answer: &BaseJoinAnswer, moved: Option<&str>) -> Result<(), Report> {
    let request_ref = &run.request_ref;
    match answer {
        BaseJoinAnswer::Plain(JoinAnswer::Rebased { from, to, .. }) => {
            let done = format!("rebased: {request_ref} moved from {from} to {to}, replayed onto the zone");
            let after = read_pending_move(&run.git, &run.folder.common, request_ref)
                .and_then(|found| found.map_or(Ok(None), |(_, base)| base));
            rewrite_then_move(run, after, (from, to), done, moved)
        }
        BaseJoinAnswer::ZoneJoined { zone_ref, from, request_from, to, .. } => {
            let done = format!(
                "joined: {zone_ref} moved from {from} to {to}; {request_ref} moved from {request_from} to {to}"
            );
            let on_zone = RecordedBase { base: from.clone(), partners: Vec::new() };
            rewrite_then_move(run, Ok(Some(on_zone)), (request_from, to), done, moved)
        }
        BaseJoinAnswer::Rebased { from, to, base, .. } => {
            let onto =
                if base.partners.is_empty() { "the zone".to_string() } else { format!("a new merge {}", base.base) };
            let done = format!("rebased: {request_ref} moved from {from} to {to}, replayed onto {onto}");
            rewrite_then_move(run, Ok(Some(recorded_base_of(base))), (from, to), done, moved)
        }
        BaseJoinAnswer::PartnersConflicted { zone, files } => {
            Err(partners_conflicted_report(&run.target.zone, zone, files, moved))
        }
        BaseJoinAnswer::Plain(_) => Ok(()),
    }
}

/// Rewrites the folder record to `base` when there is one (rewrite_record_to, its failure report after `done`, what
/// join's op moved), then moves the folder from `from` to `to`; a refused move is move_refusal_report with `done`.
fn rewrite_then_move(
    run: &RequestRun,
    base: Result<Option<RecordedBase>, String>,
    (from, to): (&Oid, &Oid),
    done: String,
    moved: Option<&str>,
) -> Result<(), Report> {
    rewrite_record_to(run, VERB, base, Some(&done), moved)?;
    move_folder(&run.git, &run.folder.toplevel, from, to, true, &run.target.actor, None)
        .map_err(|error| move_refusal_report(&error, vec![done], moved))
}
