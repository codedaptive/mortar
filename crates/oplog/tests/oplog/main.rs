//! Real-repository integration joins for oplog apply/recover and filesystem locks, plus the public JSON parser.

#[path = "../support/mod.rs"]
mod support;

mod apply;
mod attribution;
mod crash;
mod json;
mod locks;
mod marked_lines;
mod open_intent_moves;
mod open_marked_intents;
mod protection;
mod takeover_guards;
mod with_lock;
mod undo_restore;
mod recovery;
mod ref_names;
mod restore_ref_names;
mod recoverable;
mod refined_recovery;
mod remote_close;
mod remote_close_record;
mod remote_intents;
mod remote_verdict;
mod valued_lines;
mod values;
mod values_recover;
