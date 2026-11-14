//! WO-orc-snapshot-it-1: snapshot/is_saved joined to real gitio trees and oplog transactions.
//! SPEC-snapshot §1–2, S-19, S-25 and S-31. Every parent owns a disposable repository.
#[path = "../support/mod.rs"]
mod support;
mod lifecycle;
mod sparse_excludes;
mod caps;
mod race;
