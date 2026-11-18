//! temp_path: the sibling `.<file name>.<token as 16 lowercase hex>.mortar-install`.

use std::path::{Path, PathBuf};

use super::temp_path;

#[test]
fn the_token_is_sixteen_lowercase_hex_digits_padded_with_zeros() {
    assert_eq!(
        temp_path(Path::new("/h/.local/bin/mortar"), 0x1f),
        PathBuf::from("/h/.local/bin/.mortar.000000000000001f.mortar-install")
    );
}

#[test]
fn token_zero_and_the_largest_token_keep_sixteen_digits() {
    assert_eq!(
        temp_path(Path::new("/h/.local/bin/mr"), 0),
        PathBuf::from("/h/.local/bin/.mr.0000000000000000.mortar-install")
    );
    assert_eq!(
        temp_path(Path::new("/h/.local/bin/mr"), u64::MAX),
        PathBuf::from("/h/.local/bin/.mr.ffffffffffffffff.mortar-install")
    );
}

#[test]
fn the_temp_is_a_sibling_in_the_same_folder() {
    assert_eq!(
        temp_path(Path::new("/h/.claude/skills/mortar/SKILL.md"), 0xabcdef0123456789),
        PathBuf::from("/h/.claude/skills/mortar/.SKILL.md.abcdef0123456789.mortar-install")
    );
    assert_eq!(
        temp_path(Path::new("/h/.local/state/mortar/install.json"), 0x10),
        PathBuf::from("/h/.local/state/mortar/.install.json.0000000000000010.mortar-install")
    );
}
