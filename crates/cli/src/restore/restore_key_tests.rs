//! restore_key: --key wins; else a stable "restore-<16 hex>" key over the verb, the common dir, the op id and the refs.
//! The expected keys are FNV-1a 64 over those fields, each ended by a NUL, worked outside the code.

use std::path::Path;

use super::restore_key;

fn refs(names: &[&str]) -> Vec<String> {
    names.iter().map(ToString::to_string).collect()
}

#[test]
fn the_flag_is_the_key() {
    assert_eq!(restore_key(Some("k-1"), Path::new("/r/.git"), "op-1", &refs(&["refs/heads/zone/core"])), "k-1");
}

#[test]
fn the_derived_key_is_fnv_over_verb_common_op_id_and_refs() {
    let common = Path::new("/r/.git");
    assert_eq!(restore_key(None, common, "op-1", &refs(&["refs/heads/zone/core"])), "restore-f25b7736d685f855");
    assert_eq!(
        restore_key(None, common, "op-1", &refs(&["refs/heads/zone/core", "refs/heads/req/core/fix-1"])),
        "restore-fd92587fd7c27f88"
    );
}

#[test]
fn a_repeat_gives_the_same_key_and_any_input_changed_gives_another() {
    let key = |common: &str, op: &str, names: &[&str]| restore_key(None, Path::new(common), op, &refs(names));
    let base = key("/r/.git", "op-1", &["refs/heads/zone/core", "refs/heads/req/core/a"]);
    assert_eq!(base, key("/r/.git", "op-1", &["refs/heads/zone/core", "refs/heads/req/core/a"]));
    for other in [
        key("/s/.git", "op-1", &["refs/heads/zone/core", "refs/heads/req/core/a"]),
        key("/r/.git", "op-2", &["refs/heads/zone/core", "refs/heads/req/core/a"]),
        key("/r/.git", "op-1", &["refs/heads/req/core/a", "refs/heads/zone/core"]),
        key("/r/.git", "op-1", &["refs/heads/zone/core"]),
    ] {
        assert_ne!(base, other);
    }
}
