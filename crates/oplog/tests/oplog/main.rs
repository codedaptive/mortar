//! Real-repository integration joins for oplog apply/recover and filesystem locks.

// Reuse the existing real-git fixture rather than copy its temp-directory lifecycle.
#[path = "../../../gitio/tests/support/mod.rs"]
mod repository;
#[path = "../support/mod.rs"]
mod support;

mod apply;
mod crash;
mod locks;
mod recovery;
