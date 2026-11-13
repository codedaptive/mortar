//! commit_tree_request: the git run that writes a commit object from a tree.

use crate::no_nul::no_nul;
use crate::{GitError, Oid, Person, RunOpts};
use std::ffi::OsString;

/// `commit-tree <tree> -p <parent>...` with the message on stdin as given. An author or committer sets
/// GIT_<ROLE>_NAME, _EMAIL and, when its date is Some, _DATE; None sets none of them, so git's own identity applies.
/// A NUL in the message or in any person field is refused naming it: git refuses one in a message, and none can
/// pass through the environment.
pub(crate) fn commit_tree_request(
    tree: &Oid,
    parents: &[Oid],
    message: &str,
    author: Option<&Person>,
    committer: Option<&Person>,
) -> Result<(Vec<OsString>, RunOpts), GitError> {
    no_nul("commit message", message.as_bytes())?;
    let mut args: Vec<OsString> = vec!["commit-tree".into(), tree.as_str().into()];
    for parent in parents {
        args.push("-p".into());
        args.push(parent.as_str().into());
    }
    let mut env = Vec::new();
    for (role, person) in [("AUTHOR", author), ("COMMITTER", committer)] {
        if let Some(person) = person {
            identity_env(role, person, &mut env)?;
        }
    }
    Ok((
        args,
        RunOpts {
            stdin: Some(message.as_bytes().to_vec()),
            env,
            ..RunOpts::default()
        },
    ))
}

/// Appends GIT_<role>_NAME, _EMAIL and (when Some) _DATE, refusing a field holding a NUL.
fn identity_env(role: &str, person: &Person, env: &mut Vec<(OsString, OsString)>) -> Result<(), GitError> {
    let fields = [("NAME", Some(&person.name)), ("EMAIL", Some(&person.email)), ("DATE", person.date.as_ref())];
    for (field, value) in fields {
        let Some(value) = value else { continue };
        let key = format!("GIT_{role}_{field}");
        no_nul(&key, value.as_bytes())?;
        env.push((key.into(), value.into()));
    }
    Ok(())
}
