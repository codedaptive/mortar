//! clean_replay_move: what a clean replay of a zone does, from its record read. clean_replay_move_tests covers it.

use crate::after_publish::{RecordRead, ZoneMove};
use gitio::Oid;
use oplog::RefMove;
use std::ops::ControlFlow;

/// Zone `zone` at `tip`, replayed cleanly to `new`: Continue with `new` and the record move when the replay owes
/// one (Move) or none (NotOwed). Unmappable breaks with ZoneMove::ReplayDropped: the zone is left at `tip` and no op
/// runs, never a guessed record, until its own squash publishes and moves it (S-120).
pub(crate) fn clean_replay_move(
    zone: &str,
    tip: &Oid,
    new: Oid,
    read: RecordRead,
) -> ControlFlow<ZoneMove, (Oid, Option<RefMove>)> {
    match read {
        RecordRead::NotOwed => ControlFlow::Continue((new, None)),
        RecordRead::Move(record) => ControlFlow::Continue((new, Some(record))),
        RecordRead::Unmappable { source } => {
            ControlFlow::Break(ZoneMove::ReplayDropped { zone: zone.to_string(), tip: tip.clone(), squash_of: source })
        }
    }
}
