//! ReplayGit for gitio::Git: each method calls gitio's inherent method of the same name; replay passes onto as
//! gitio's attr_source. No unit test: it only runs git; Nagatha's integration tests of `replay` are to cover it.

use crate::replay_git::ReplayGit;
use gitio::{CommitInfo, Git, GitError, MergeTreeOutcome, Oid, Person, ReplayOutcome};

impl ReplayGit for Git {
    fn replay_available(&self) -> Result<bool, GitError> {
        Git::replay_available(self)
    }

    /// Passes `onto` as gitio's `attr_source`: join always reads merge attributes from onto.
    fn replay(&self, onto: &Oid, base: &Oid, tip_ref: &str) -> Result<ReplayOutcome, GitError> {
        Git::replay(self, onto, onto, base, tip_ref)
    }

    fn rev_parse(&self, rev: &str) -> Result<Option<Oid>, GitError> {
        Git::rev_parse(self, rev)
    }

    fn read_ref(&self, refname: &str) -> Result<Option<Oid>, GitError> {
        Git::read_ref(self, refname)
    }

    fn rev_list(&self, base: &Oid, tip: &Oid) -> Result<Vec<Oid>, GitError> {
        Git::rev_list(self, base, tip)
    }

    fn commit_info(&self, commit: &Oid) -> Result<CommitInfo, GitError> {
        Git::commit_info(self, commit)
    }

    fn merge_tree(
        &self,
        attr_source: &Oid,
        merge_base: Option<&Oid>,
        ours: &Oid,
        theirs: &Oid,
    ) -> Result<MergeTreeOutcome, GitError> {
        Git::merge_tree(self, attr_source, merge_base, ours, theirs)
    }

    fn commit_tree(
        &self,
        tree: &Oid,
        parents: &[Oid],
        message: &str,
        author: Option<&Person>,
        committer: Option<&Person>,
    ) -> Result<Oid, GitError> {
        Git::commit_tree(self, tree, parents, message, author, committer)
    }
}
