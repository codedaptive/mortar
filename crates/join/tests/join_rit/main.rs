//! WO-join-rit-3 / #1614: public resolve -> real gitio typed readers, conversions and refusal Display.
//! RIT only: no unit/ZIT target is included. Each parent owns a disposable, config-isolated TestRepo.
//! S-109: no private API is exposed. Two extracted inputs cannot reach the private helper from this surface:
//! tree_entry verifies the response path before constructing NotABlob; typed Overlap names paths from its input.
//! Their public equivalents reject a wrong-path commit record and name an earlier-round overlap from all_files,
//! even when neither path is in current files. They do NOT claim a mismatched NotABlob or path=None Overlap.
//! State fixtures are literal on-disk reader inputs with real objects/refs, not private state/codec inclusion.

mod classify;
mod support;
mod tree_refusals;
