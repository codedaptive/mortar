//! ReplayGit: the git calls replay makes, as a trait so its decisions can be unit-tested with a fake
//! (replay_git_fake). A declaration only; git_replay_git implements it for gitio::Git.

use gitio::{CommitInfo, GitError, MergeTreeOutcome, Oid, Person, ReplayOutcome};
use std::path::PathBuf;

/// gitio's methods of the same names and signatures, except replay.
pub(crate) trait ReplayGit {
    fn replay_available(&self) -> Result<bool, GitError>;
    /// gitio's replay without its attr_source: join always reads merge attributes from onto.
    fn replay(&self, onto: &Oid, base: &Oid, tip_ref: &str) -> Result<ReplayOutcome, GitError>;
    fn rev_parse(&self, rev: &str) -> Result<Option<Oid>, GitError>;
    fn read_ref(&self, refname: &str) -> Result<Option<Oid>, GitError>;
    fn rev_list(&self, base: &Oid, tip: &Oid) -> Result<Vec<Oid>, GitError>;
    fn commit_info(&self, commit: &Oid) -> Result<CommitInfo, GitError>;
    fn merge_tree(
        &self,
        attr_source: &Oid,
        merge_base: Option<&Oid>,
        ours: &Oid,
        theirs: &Oid,
    ) -> Result<MergeTreeOutcome, GitError>;
    fn commit_tree(
        &self,
        tree: &Oid,
        parents: &[Oid],
        message: &str,
        author: Option<&Person>,
        committer: Option<&Person>,
    ) -> Result<Oid, GitError>;
    fn tree_with_paths(&self, base: &Oid, from: &Oid, paths: &[PathBuf]) -> Result<Oid, GitError>;
}
