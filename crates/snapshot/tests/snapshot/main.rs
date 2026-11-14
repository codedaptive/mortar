//! WO-orc-snapshot-it-1: snapshot/is_saved joined to real gitio trees and oplog transactions.
//! SPEC-snapshot WO-snapshot-1, WO-snapshot-2, Tests; S-19, S-25 and S-31. Every parent owns a disposable repository.
#[path = "../support/mod.rs"]
mod support;
mod lifecycle;
mod sparse_excludes;
mod caps;
mod race;
mod formatter;
mod formatter_paths;
mod formatter_timeout;
