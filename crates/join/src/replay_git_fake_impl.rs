//! ReplayGit for ReplayGitFake: answers each call from the fake's scripted fields and logs it with its arguments.
//! Test support, declared test-only. A call with no scripted answer panics naming it (read_ref alone answers None),
//! so a test sees a call it did not expect.

use crate::fake_call::FakeCall;
use crate::replay_git::ReplayGit;
use crate::replay_git_fake::ReplayGitFake;
use gitio::{CommitInfo, GitError, MergeTreeOutcome, Oid, Person, ReplayOutcome, TreeWithPathsError};
use std::path::PathBuf;

impl ReplayGit for ReplayGitFake {
    fn replay_available(&self) -> Result<bool, GitError> {
        self.calls.borrow_mut().push(FakeCall::ReplayAvailable);
        Ok(self.available.expect("unscripted replay_available"))
    }

    fn replay(&self, onto: &Oid, base: &Oid, tip_ref: &str) -> Result<ReplayOutcome, GitError> {
        self.calls.borrow_mut().push(FakeCall::Replay {
            onto: onto.clone(),
            base: base.clone(),
            tip_ref: tip_ref.to_string(),
        });
        Ok(self.replay.clone().expect("unscripted replay"))
    }

    fn rev_parse(&self, rev: &str) -> Result<Option<Oid>, GitError> {
        self.calls.borrow_mut().push(FakeCall::RevParse(rev.to_string()));
        self.rev_parse.clone().expect("unscripted rev_parse")
    }

    fn read_ref(&self, refname: &str) -> Result<Option<Oid>, GitError> {
        self.calls.borrow_mut().push(FakeCall::ReadRef(refname.to_string()));
        Ok(self.refs.get(refname).cloned())
    }

    fn rev_list(&self, base: &Oid, tip: &Oid) -> Result<Vec<Oid>, GitError> {
        self.calls.borrow_mut().push(FakeCall::RevList { base: base.clone(), tip: tip.clone() });
        Ok(self.rev_list.clone().expect("unscripted rev_list"))
    }

    fn commit_info(&self, commit: &Oid) -> Result<CommitInfo, GitError> {
        self.calls.borrow_mut().push(FakeCall::CommitInfo(commit.clone()));
        Ok(self.commits.get(commit).cloned().expect("unscripted commit_info"))
    }

    fn merge_tree(
        &self,
        attr_source: &Oid,
        merge_base: Option<&Oid>,
        ours: &Oid,
        theirs: &Oid,
    ) -> Result<MergeTreeOutcome, GitError> {
        self.calls.borrow_mut().push(FakeCall::MergeTree {
            attr_source: attr_source.clone(),
            merge_base: merge_base.cloned(),
            ours: ours.clone(),
            theirs: theirs.clone(),
        });
        Ok(self.merges.get(theirs).cloned().expect("unscripted merge_tree"))
    }

    fn commit_tree(
        &self,
        tree: &Oid,
        parents: &[Oid],
        message: &str,
        author: Option<&Person>,
        committer: Option<&Person>,
    ) -> Result<Oid, GitError> {
        self.calls.borrow_mut().push(FakeCall::CommitTree {
            tree: tree.clone(),
            parents: parents.to_vec(),
            message: message.to_string(),
            author: author.cloned(),
            committer: committer.cloned(),
        });
        Ok(self.new_commits.borrow_mut().pop_front().expect("unscripted commit_tree"))
    }

    fn tree_with_paths_typed(&self, base: &Oid, from: &Oid, paths: &[PathBuf]) -> Result<Oid, TreeWithPathsError> {
        self.calls.borrow_mut().push(FakeCall::TreeWithPaths {
            base: base.clone(),
            from: from.clone(),
            paths: paths.to_vec(),
        });
        self.tree_with_paths.clone().expect("unscripted tree_with_paths_typed")
    }
}
