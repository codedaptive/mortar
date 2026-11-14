//! push_request: the git run that pushes a list of refs to one remote.

use crate::text_refusal::text_refusal;
use crate::usable_ref_arg::usable_ref_arg;
use crate::usable_remote::usable_remote;
use crate::{GitError, PushLease, PushRef, RunOpts};
use std::ffi::OsString;

/// `push --porcelain --no-follow-tags [--force-with-lease=<refname>:<expected>]... --end-of-options <remote>
/// <new>:<refname> | :<refname> ...`. `--porcelain` gives one machine-readable record per ref for read_push.
/// `--no-follow-tags` keeps a user's push.followTags from adding tag records nobody asked for. Each leased ref gets
/// its own `--force-with-lease`, in `refs` order: `<refname>:<oid>` for At, `<refname>:` (must not exist) for Absent;
/// Unleased adds none. Exit 1 (some ref refused, or the whole push refused) is an answer read_push tells apart.
/// Refused before running: no refs (git with no refspec would push its configured default), an unusable remote or
/// ref name, and a ref named twice (git would refuse the whole push).
pub(crate) fn push_request(remote: &str, refs: &[PushRef]) -> Result<(Vec<OsString>, RunOpts), GitError> {
    if refs.is_empty() {
        return Err(GitError::refusal(Vec::new(), "push needs at least one ref"));
    }
    usable_remote(remote)?;
    for (at, push_ref) in refs.iter().enumerate() {
        usable_ref_arg("ref name", &push_ref.refname)?;
        if refs[..at].iter().any(|earlier| earlier.refname == push_ref.refname) {
            return Err(text_refusal("push names a ref twice", &push_ref.refname));
        }
    }
    let mut args: Vec<OsString> = vec!["push".into(), "--porcelain".into(), "--no-follow-tags".into()];
    args.extend(refs.iter().filter_map(lease_arg));
    args.extend(["--end-of-options".into(), remote.into()]);
    args.extend(refs.iter().map(refspec));
    Ok((
        args,
        RunOpts {
            answer_codes: vec![1],
            ..RunOpts::default()
        },
    ))
}

fn lease_arg(push_ref: &PushRef) -> Option<OsString> {
    match &push_ref.lease {
        PushLease::Unleased => None,
        PushLease::Absent => Some(format!("--force-with-lease={}:", push_ref.refname).into()),
        PushLease::At(oid) => Some(format!("--force-with-lease={}:{oid}", push_ref.refname).into()),
    }
}

fn refspec(push_ref: &PushRef) -> OsString {
    match &push_ref.new {
        Some(oid) => format!("{oid}:{}", push_ref.refname).into(),
        None => format!(":{}", push_ref.refname).into(),
    }
}
