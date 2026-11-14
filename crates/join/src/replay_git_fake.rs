//! ReplayGitFake: a scripted ReplayGit for unit tests; replay_git_fake_impl implements the trait. Test support,
//! declared test-only. Each answer is set in a field; every call is logged in `calls`, in order, with its arguments.

use crate::fake_call::FakeCall;
use gitio::{CommitInfo, GitError, MergeTreeOutcome, Oid, ReplayOutcome, TreeWithPathsError};
use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};

#[derive(Default)]
pub(crate) struct ReplayGitFake {
    pub(crate) available: Option<bool>,
    pub(crate) replay: Option<ReplayOutcome>,
    /// rev_parse's answer, whatever the rev.
    pub(crate) rev_parse: Option<Result<Option<Oid>, GitError>>,
    /// read_ref answers None for a ref not in the map.
    pub(crate) refs: HashMap<String, Oid>,
    pub(crate) rev_list: Option<Vec<Oid>>,
    pub(crate) commits: HashMap<Oid, CommitInfo>,
    /// merge_tree's answer, keyed by `theirs`.
    pub(crate) merges: HashMap<Oid, MergeTreeOutcome>,
    /// commit_tree's answers, one per call, in order.
    pub(crate) new_commits: RefCell<VecDeque<Oid>>,
    /// tree_with_paths_typed's answer, whatever its arguments.
    pub(crate) tree_with_paths: Option<Result<Oid, TreeWithPathsError>>,
    pub(crate) calls: RefCell<Vec<FakeCall>>,
}
