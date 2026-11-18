//! find_harness: each harness by its exact name; anything else is None.

use super::find_harness;

#[test]
fn each_name_finds_its_harness() {
    assert_eq!(find_harness("claude").map(|h| h.folder), Some(".claude"));
    assert_eq!(find_harness("codex").map(|h| h.skills), Some(".agents/skills"));
    assert_eq!(find_harness("cursor").map(|h| h.folder), Some(".cursor"));
}

#[test]
fn a_name_that_is_not_exact_finds_nothing() {
    for name in ["", "Claude", "claude ", "clau", "agents", ".claude", "mcp"] {
        assert_eq!(find_harness(name), None, "{name:?}");
    }
}
