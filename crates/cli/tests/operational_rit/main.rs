//! WO-cli-rit-4/#1638: actual mortar children -> Status/Need/Guard -> real lower kits.
//! Disposable repositories only; fixed data and process/file events, never a speed measurement.
//! Shared fixture/parser sources are read-only; no legacy tests or lane ZITs are included.
#[allow(dead_code)]
#[path = "../cli_rit/json.rs"]
mod json;
#[allow(dead_code)]
#[path = "../cli_rit/support.rs"]
mod support;
mod observe;
mod need_fixture;
mod cid;
mod refs;
mod guard_fixture;
mod status;
mod need;
mod guard;
mod handoff;
