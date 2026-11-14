//! Real-repository integration joins for oplog apply/recover and filesystem locks.

#[path = "../support/mod.rs"]
mod support;

mod apply;
mod crash;
mod locks;
mod protection;
mod takeover_guards;
mod undo_restore;
mod recovery;
mod ref_names;
mod refined_recovery;
