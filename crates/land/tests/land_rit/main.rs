//! Land RITs: published WO-orc-land-rit-1 interfaces and WO-land-rit-2's owed cross-kit cases.
//! Real Gitio/Oplog Display/JSON/trailers through public Land APIs; every parent owns a disposable TestRepo.
//! WO-land-rit-3: after_publish -> Gitio/Oplog metadata/Display, and S-103 actual interrupted publish -> recover/repeat.
//! push_kill's ignored entry is a subprocess only; its parents prove actual SIGKILL and kernel reap.
mod displays;
mod support;
mod trailers;
mod owed_displays;
mod guard_codec;
mod owed_ops;
mod state_codec;
mod after_publish_displays;
mod after_publish_ops;
mod push_fixture;
mod push_kill;
mod remote_intents;
