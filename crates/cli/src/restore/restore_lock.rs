//! The one lock a `mortar restore` holds, chosen from the refs it names before any lock or git run (S-113(1),
//! S-115(2)).

use oplog::LockName;

/// Whose a ref is: the landing refs' or one zone's.
#[derive(Debug, PartialEq, Eq)]
enum Owner<'a> {
    Landing,
    Zone(&'a str),
}

/// Every ref one zone's takes `join-<zone>`; every ref a landing ref (exactly refs/heads/<integration> or
/// refs/heads/<origin>, checked first) takes `land`. A zone's refs are refs/heads/req/<z>/<rest>,
/// refs/heads/zone/<z>, refs/mortar/snap/<z>/<rest>, refs/heads/try/<z>/<rest> and refs/mortar/trysnap/<z>/<rest>
/// (S-69), with <z> and <rest> non-empty. Err (a usage reason) for no refs, for the first ref that is neither, and
/// for a landing ref with a zone's ref or refs of two zones, naming the first ref of each side.
pub fn restore_lock(refs: &[String], origin_branch: &str, integration_branch: &str) -> Result<LockName, String> {
    let landing = [format!("refs/heads/{integration_branch}"), format!("refs/heads/{origin_branch}")];
    let mut owners = Vec::with_capacity(refs.len());
    for refname in refs {
        let owner = if landing.contains(refname) { Some(Owner::Landing) } else { zone_of(refname).map(Owner::Zone) };
        let Some(owner) = owner else {
            return Err(format!(
                "{refname} is neither a zone's ref (refs/heads/req/<zone>/..., refs/heads/zone/<zone>, \
                 refs/heads/try/<zone>/..., refs/mortar/snap/<zone>/..., refs/mortar/trysnap/<zone>/...) nor a \
                 landing ref ({}, {})",
                landing[0], landing[1]
            ));
        };
        owners.push((refname, owner));
    }
    let Some((first, first_owner)) = owners.first() else {
        return Err("restore needs at least one full ref name after the op id".to_string());
    };
    if let Some((other, other_owner)) = owners.iter().find(|(_, owner)| owner != first_owner) {
        return Err(mixed(first, first_owner, other, other_owner));
    }
    Ok(match first_owner {
        Owner::Landing => LockName::Land,
        Owner::Zone(zone) => LockName::Join((*zone).to_string()),
    })
}

/// The zone of a zone's ref, or None.
fn zone_of(refname: &str) -> Option<&str> {
    if let Some(zone) = refname.strip_prefix("refs/heads/zone/") {
        return (!zone.is_empty() && !zone.contains('/')).then_some(zone);
    }
    ["refs/heads/req/", "refs/mortar/snap/", "refs/heads/try/", "refs/mortar/trysnap/"].iter().find_map(|prefix| {
        let (zone, rest) = refname.strip_prefix(prefix)?.split_once('/')?;
        (!zone.is_empty() && !rest.is_empty()).then_some(zone)
    })
}

fn mixed(first: &str, first_owner: &Owner, other: &str, other_owner: &Owner) -> String {
    let whose = |refname: &str, owner: &Owner| match owner {
        Owner::Landing => format!("{refname} is a landing ref"),
        Owner::Zone(zone) => format!("{refname} is zone {zone}'s"),
    };
    format!(
        "{} and {}: restore one zone's refs, or the landing refs, per call",
        whose(first, first_owner),
        whose(other, other_owner)
    )
}
