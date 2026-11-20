//! Test conversions shared by the crate's src tests: hex to Oid, (hex, refname) pairs to the (Oid, refname) list
//! git's ref listing gives, (refname, hex) pairs to the (refname, Oid) list a sweep picks, and &str lists to owned
//! strings.

mod oid;
mod refs;
mod picked;
mod strings;

pub(crate) use oid::oid;
pub(crate) use refs::refs;
pub(crate) use picked::picked;
pub(crate) use strings::strings;
