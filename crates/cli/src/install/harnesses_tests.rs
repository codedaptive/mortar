//! HARNESSES: the table pinned to spec 07 items 4 and 5 (names, detecting folders, skills folders, order).

use super::HARNESSES;

#[test]
fn the_table_is_claude_codex_cursor_with_their_folders_in_that_order() {
    let rows: Vec<(&str, &str, &str)> = HARNESSES.iter().map(|h| (h.name, h.folder, h.skills)).collect();
    assert_eq!(
        rows,
        vec![
            ("claude", ".claude", ".claude/skills"),
            ("codex", ".codex", ".agents/skills"),
            ("cursor", ".cursor", ".cursor/skills"),
        ]
    );
}
