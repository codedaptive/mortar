//! guard_decide: whether a git hook may let a move of origin or integration through (SPEC-land 06, S-60).

use gitio::Oid;
use oplog::RefMove;

use super::{GuardHook, GuardVerdict};
use crate::LandRefs;

const OWNER: &str = "the landing surface: mr land and mr publish";
const NEXT: &str = "bring your work in with `mr join`, or ask for a landing";

/// Decides one hook run from its stdin `input`. `op_moves` is the moves of the open intent MORTAR_OP names, None
/// when MORTAR_OP is unset or names no open intent: the env var alone never allows anything, since anyone can set
/// it (S-60). A move of `refs/heads/<origin_branch>` or `refs/heads/<integration_branch>` is allowed only when the
/// intent lists that ref with the same old and new; every other readable line is allowed, and a line the guard cannot
/// read is refused. The first line refused decides.
///
/// A pre-push is read only when it is to the guarded remote `refs.remote` (S-100): its first argument is that name,
/// or its URL is exactly one of the push URLs the caller read for it (text compared as is: no folding of case,
/// trailing slash, `.git` or ssh against https, since the guard stops accidental moves and is no defence against
/// someone working round it). A push by that name is decided without the URLs. A push under another name when the
/// caller could not read those URLs (`push_urls` None) is refused whole, unread: the guard cannot tell where it
/// goes, so it fails closed, as it does on input it cannot read.
pub fn guard_decide(hook: &GuardHook, input: &str, refs: &LandRefs, op_moves: Option<&[RefMove]>) -> GuardVerdict {
    let guarded = [format!("refs/heads/{}", refs.origin_branch), format!("refs/heads/{}", refs.integration_branch)];
    let moves = op_moves.unwrap_or_default();
    let lines = || input.split('\n').filter(|line| !line.is_empty());
    let push_lines = || lines().find_map(|line| push_refusal(line, &refs.remote, &guarded, moves));
    let why = match hook {
        // prepared is the state in which every ref is locked and git still aborts on a non-zero exit, so one check
        // there covers each transaction; the other states are let through unread.
        GuardHook::ReferenceTransaction { state } if state == "prepared" => {
            lines().find_map(|line| transaction_refusal(line, &guarded, moves))
        }
        GuardHook::PrePush { remote, .. } if *remote == refs.remote => push_lines(),
        GuardHook::PrePush { url, push_urls: Some(urls), .. } => urls.contains(url).then(push_lines).flatten(),
        GuardHook::PrePush { url, push_urls: None, .. } => Some(format!(
            "cannot read {0}'s push URLs, so the guard cannot tell whether the push to {url} is to {0}",
            refs.remote
        )),
        _ => None,
    };
    match why {
        None => GuardVerdict::Allow,
        Some(why) => GuardVerdict::Refuse { why, owner: OWNER.to_string(), next: NEXT.to_string() },
    }
}

/// A reference-transaction line, `<old> SP <new> SP <refname>`: the refusal's why, or None when it may pass.
fn transaction_refusal(line: &str, guarded: &[String], moves: &[RefMove]) -> Option<String> {
    let Some([old, new, refname]) = read_line(line) else {
        return Some(unreadable("reference-transaction", line));
    };
    let (Ok(old), Ok(new)) = (Oid::parse(old), Oid::parse(new)) else {
        return Some(unreadable("reference-transaction", line));
    };
    if !guarded.iter().any(|g| g == refname) || intended(moves, refname, old, new) {
        return None;
    }
    Some(format!("only Mortar moves {refname}"))
}

/// A pre-push line, `<local ref> SP <local oid> SP <remote ref> SP <remote oid>`; the remote ref is the one moved,
/// from the remote oid to the local oid. `(delete)` as the local ref comes with an all-zero local oid. `remote` is
/// the guarded remote's name.
fn push_refusal(line: &str, remote: &str, guarded: &[String], moves: &[RefMove]) -> Option<String> {
    let Some([_, local_oid, remote_ref, remote_oid]) = read_line(line) else {
        return Some(unreadable("pre-push", line));
    };
    let (Ok(new), Ok(old)) = (Oid::parse(local_oid), Oid::parse(remote_oid)) else {
        return Some(unreadable("pre-push", line));
    };
    if !guarded.iter().any(|g| g == remote_ref) || intended(moves, remote_ref, old, new) {
        return None;
    }
    Some(format!("only Mortar pushes {remote_ref} to {remote}"))
}

/// Exactly N space-separated, non-empty fields; anything else is unreadable.
fn read_line<const N: usize>(line: &str) -> Option<[&str; N]> {
    let fields: [&str; N] = line.split(' ').collect::<Vec<_>>().try_into().ok()?;
    fields.iter().all(|f| !f.is_empty()).then_some(fields)
}

/// Whether the intent lists exactly this move. Git writes an all-zero old for a ref that does not exist yet, which
/// a RefMove says with `old: None`; an all-zero new is a deletion and must match the intent's zero `new`.
fn intended(moves: &[RefMove], refname: &str, old: Oid, new: Oid) -> bool {
    let old = (!old.as_str().bytes().all(|b| b == b'0')).then_some(old);
    moves.iter().any(|m| m.refname == refname && m.old == old && m.new == new)
}

/// A line the guard cannot read is refused: it cannot tell what the line moves, so it fails closed.
fn unreadable(hook: &str, line: &str) -> String {
    format!("cannot read the {hook} hook's input line {line:?}")
}
