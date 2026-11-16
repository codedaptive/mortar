//! Test conversions shared by the folders integration target: hex to Oid, (hex, refname) pairs to the
//! (Oid, refname) list git's ref listing gives, (refname, hex) pairs to the (refname, Oid) list a sweep picks, and
//! &str lists to owned strings. The crate's src tests use src/test_ref_pairs/, the same four in a cfg(test)
//! module, because a tests/ target cannot reach a src cfg(test) module.

mod oid;
mod refs;
mod picked;
mod strings;

pub use oid::oid;
pub use refs::refs;
pub use picked::picked;
pub use strings::strings;
