//! Reads a ref's current value through git.
//! No unit test: it runs git (S-5); ref_value_from_output_tests covers reading the answer. Nagatha's
//! integration group, in a throwaway repository, should cover: an existing ref returns its object id; a
//! missing ref returns None; a git that cannot run is OplogError::Git.

use gitio::{Git, Oid, RunOpts};

use super::{ref_value_from_output, OplogError};

/// The object id `refname` holds, or None when the ref does not exist.
pub fn read_ref_value(git: &Git, refname: &str) -> Result<Option<Oid>, OplogError> {
    let opts = RunOpts {
        answer_codes: vec![1],
        read_only: true,
        ..RunOpts::default()
    };
    let out = git
        .run(&["rev-parse", "-q", "--verify", refname], &opts)
        .map_err(OplogError::Git)?;
    ref_value_from_output(out.status, &out.stdout)
}
