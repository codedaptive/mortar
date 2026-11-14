//! WO-gitio-18: Git::remote_push_urls, the URLs `git push` would use for a remote, and its typed refusals. Zone
//! integration tests: real git in throwaway repositories made by tests/support's GitRepo; expected values are
//! literals written from the contract.
#[path = "../support/mod.rs"]
mod support;
mod fixture;
mod odd_urls;
mod refusals;
mod urls;
