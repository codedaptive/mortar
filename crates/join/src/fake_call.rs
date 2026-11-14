//! FakeCall: one call the fake ReplayGit received, with its arguments. Test support, declared test-only.

use gitio::{Oid, Person};

/// The call and the arguments it was given, so tests can assert their order and values.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum FakeCall {
    ReplayAvailable,
    Replay { onto: Oid, base: Oid, tip_ref: String },
    RevParse(String),
    ReadRef(String),
    RevList { base: Oid, tip: Oid },
    CommitInfo(Oid),
    MergeTree { attr_source: Oid, merge_base: Option<Oid>, ours: Oid, theirs: Oid },
    CommitTree { tree: Oid, parents: Vec<Oid>, message: String, author: Option<Person>, committer: Option<Person> },
}
