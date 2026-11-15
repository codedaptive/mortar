//! restore_lock: which lock the refs named choose, and which sets of refs are a usage error (S-113(1), S-115(2)).

use oplog::LockName;

use super::restore_lock;

fn lock(refs: &[&str]) -> Result<LockName, String> {
    let refs: Vec<String> = refs.iter().map(ToString::to_string).collect();
    restore_lock(&refs, "main", "integration")
}

fn join(zone: &str) -> Result<LockName, String> {
    Ok(LockName::Join(zone.to_string()))
}

fn refused_naming(refs: &[&str], named: &[&str]) {
    let reason = lock(refs).expect_err("a usage error");
    for name in named {
        assert!(reason.contains(name), "{reason:?} does not name {name}");
    }
}

#[test]
fn each_kind_of_zone_ref_takes_that_zones_join_lock() {
    assert_eq!(lock(&["refs/heads/req/core/fix-1"]), join("core"));
    assert_eq!(lock(&["refs/heads/zone/core"]), join("core"));
    assert_eq!(lock(&["refs/mortar/snap/core/fix-1"]), join("core"));
    assert_eq!(lock(&["refs/heads/try/core/fix-1/2"]), join("core"));
    assert_eq!(lock(&["refs/mortar/trysnap/core/fix-1/2"]), join("core"));
}

#[test]
fn refs_all_of_one_zone_take_its_join_lock() {
    assert_eq!(
        lock(&["refs/heads/zone/core", "refs/heads/req/core/fix-1", "refs/mortar/snap/core/fix-1"]),
        join("core")
    );
}

#[test]
fn the_integration_and_origin_branches_take_the_land_lock() {
    assert_eq!(lock(&["refs/heads/integration"]), Ok(LockName::Land));
    assert_eq!(lock(&["refs/heads/main", "refs/heads/integration"]), Ok(LockName::Land));
}

#[test]
fn a_landing_ref_is_checked_before_the_zone_patterns() {
    let refs = vec!["refs/heads/zone/core".to_string()];
    assert_eq!(restore_lock(&refs, "main", "zone/core"), Ok(LockName::Land));
}

#[test]
fn no_refs_is_a_usage_error() {
    assert_eq!(lock(&[]), Err("restore needs at least one full ref name after the op id".to_string()));
}

#[test]
fn a_ref_of_no_zone_and_not_landing_is_refused_naming_it() {
    refused_naming(&["refs/heads/feature/x"], &["refs/heads/feature/x"]);
    refused_naming(&["refs/heads/zone/core", "refs/tags/v1"], &["refs/tags/v1"]);
}

#[test]
fn zone_refs_missing_a_part_are_refused() {
    for bad in [
        "refs/heads/zone/",
        "refs/heads/zone/core/extra",
        "refs/heads/req/core",
        "refs/heads/req/core/",
        "refs/heads/req//fix-1",
        "refs/mortar/snap/core",
        "refs/heads/try/core",
        "refs/mortar/trysnap//fix-1/1",
        "main",
        "integration",
        "refs/heads/integration/x",
    ] {
        refused_naming(&[bad], &[bad]);
    }
}

#[test]
fn refs_of_two_zones_are_refused_naming_both() {
    let refs = ["refs/heads/zone/core", "refs/heads/req/cli/fix-1"];
    refused_naming(&refs, &refs);
}

#[test]
fn a_landing_ref_with_a_zone_ref_is_refused_naming_both() {
    for refs in [["refs/heads/main", "refs/heads/zone/core"], ["refs/mortar/snap/core/a", "refs/heads/integration"]] {
        refused_naming(&refs, &refs);
    }
}

#[test]
fn an_unknown_ref_is_named_even_after_a_mix() {
    refused_naming(&["refs/heads/zone/a", "refs/heads/zone/b", "refs/heads/feature/x"], &["refs/heads/feature/x"]);
}
