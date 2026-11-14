//! WO-orc-land-rit-1 #1410: exactly the eight cross-zone cases owed in Coord-land#1383.
//! Land Display -> real Gitio/Oplog Display; public squash -> Oplog pairs -> Gitio trailers -> persisted order.
//! Lane-only squash/rereview/bisect flows remain Land ZITs. Every parent owns a disposable TestRepo.
mod displays;
mod support;
mod trailers;
