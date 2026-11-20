//! The op key a verb passes to the op log and the lock.
//! save's and undo's keys go to the lock holder only; so do snapshot's, sweep's, guard's and the landing
//! verbs' keys, since their calls draw their own op keys.
//! Shell API: `mr folder` (run_folder's runners), `mr join` and `mr resolve` (through request_run_for),
//! `mr save` and `mr undo` (through save_run_for), `mr need` (run_need), `mr guard`
//! (run_guard_install, run_guard_status), `mr squash` (run_squash),
//! `mr rereview` (run_rereview), `mr back-out` (run_back_out), `mr publish` (run_publish),
//! `mr would-merge` (run_would_merge), `mr bisect` (run_bisect), `mr backup` (run_backup),
//! `mr restore-backup` (run_restore_backup), `mr land` (run_land), `mr after-publish`
//! (run_after_publish), `mr snapshot` (run_snapshot) and `mr sweep` (run_sweep) call it, and so will each
//! later verb that moves a ref.

use std::path::Path;

use oplog::RefMove;

use super::hash_key;

/// `flag` (--key) when given. Else hash_key's "<verb>-<16 hex digits>" over the verb, the folder and each move's
/// ref, old value ("none" when the ref must not exist) and new value. Same inputs give the same key, so a plain
/// repeat of a command is the same operation.
pub fn op_key(flag: Option<&str>, verb: &str, folder: &Path, moves: &[RefMove]) -> String {
    if let Some(key) = flag {
        return key.to_string();
    }
    let mut fields: Vec<&[u8]> = vec![verb.as_bytes(), folder.as_os_str().as_encoded_bytes()];
    for one in moves {
        fields.push(one.refname.as_bytes());
        fields.push(one.old.as_ref().map_or("none", |old| old.as_str()).as_bytes());
        fields.push(one.new.as_str().as_bytes());
    }
    hash_key(verb, &fields)
}
