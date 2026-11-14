//! Whether sweep_refs may delete a ref its caller named (S-53, S-54).

use gitio::Oid;
use oplog::ProtectedRef;

use super::ExtraRef;

/// Namespaces Mortar, git and the remotes own; a caller's ref list never reaches into them (S-53, S-54).
const REFUSED_PREFIXES: [&str; 3] = ["refs/heads/", "refs/mortar/", "refs/remotes/"];

/// Refuse "is under <prefix>" for a name under refs/heads/, refs/mortar/ or refs/remotes/, then "is protected by
/// <by>" for a name in `protected`; both before the ref is looked up, so a bad name is reported even when absent.
/// Then Skip when `refs` (list_refs' shape) does not hold it, or else Accept its id.
pub fn extra_ref_verdict(refname: &str, refs: &[(Oid, String)], protected: &[ProtectedRef]) -> ExtraRef {
    if let Some(prefix) = REFUSED_PREFIXES.iter().find(|prefix| refname.starts_with(*prefix)) {
        return ExtraRef::Refuse(format!("is under {prefix}"));
    }
    if let Some(guard) = protected.iter().find(|guard| guard.refname == refname) {
        return ExtraRef::Refuse(format!("is protected by {}", guard.by));
    }
    match refs.iter().find(|(_, name)| name == refname) {
        Some((oid, _)) => ExtraRef::Accept(oid.clone()),
        None => ExtraRef::Skip,
    }
}
