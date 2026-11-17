//! pick_squash_ref: the squash ref under refs/mortar/squash/ that carries a change ID, from refs already read.

use gitio::Oid;
use oplog::ChangeId;

use super::SquashPick;

/// `squashes` holds each squash ref's full refname, its commit and the change ID read from it (None when it could
/// not be read as a squash), in git's order. Exactly one entry carrying `change` is One with its refname and commit;
/// none is Missing; two or more are Several with their refnames in input order. None never matches.
pub fn pick_squash_ref(change: &ChangeId, squashes: &[(String, Oid, Option<ChangeId>)]) -> SquashPick {
    let carrying: Vec<&(String, Oid, Option<ChangeId>)> =
        squashes.iter().filter(|(_, _, carried)| carried.as_ref() == Some(change)).collect();
    match carrying.as_slice() {
        [] => SquashPick::Missing,
        [(refname, commit, _)] => SquashPick::One { refname: refname.clone(), commit: commit.clone() },
        several => SquashPick::Several { refnames: several.iter().map(|(refname, _, _)| refname.clone()).collect() },
    }
}
