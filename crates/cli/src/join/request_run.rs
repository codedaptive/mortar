//! What a request verb (join, resolve) reads once, before its lock.
//! A bare type (S-7); request_run_for makes it, and request_run_for_tests exercises it.

use gitio::Git;
use oplog::LockName;
use snapshot::SnapshotTarget;

use crate::config::SnapshotSettings;
use crate::context::FolderContext;

/// `git` runs in the folder's top; `request_ref` is the request branch's full ref name; `lock` the verb's lock and
/// `key` its op key.
#[derive(Debug, Clone)]
pub struct RequestRun<'a> {
    pub git: Git,
    pub folder: &'a FolderContext,
    pub settings: SnapshotSettings,
    pub target: SnapshotTarget,
    pub request_ref: String,
    pub lock: LockName,
    pub key: String,
}
