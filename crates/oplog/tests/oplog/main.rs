//! Real-repository integration joins for oplog apply/recover and filesystem locks.

#[path = "../support/mod.rs"]
mod support;

mod apply;
mod attribution;
mod crash;
mod locks;
mod open_intent_moves;
mod protection;
mod takeover_guards;
mod with_lock;
mod undo_restore;
mod recovery;
mod ref_names;
mod refined_recovery;
