//! Land RITs: public interfaces and cross-kit cases.
//! Real Gitio/Oplog Display/JSON/trailers through public Land APIs; every parent owns a disposable TestRepo.
//! after_publish -> Gitio/Oplog metadata, and actual interrupted publish -> recover/repeat.
//! after_publish's consumed replay record and zone tip recovered by public Oplog undo/restore.
//! push_kill's ignored entry is a subprocess only; its parents prove actual SIGKILL and kernel reap.
mod support;
mod trailers;
mod guard_codec;
mod guard_installation;
mod check_hooks_path;
mod owed_ops;
mod state_codec;
mod after_publish_ops;
mod push_fixture;
mod push_kill;
mod remote_intents;
mod replay_record_recovery;
