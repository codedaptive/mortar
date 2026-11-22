//! draw_temp_path: the drawn name sits beside the file, carries 16 lowercase hex digits, and differs from draw to draw.

use std::collections::HashSet;
use std::path::Path;

use super::draw_temp_path::draw_temp_path;

/// The part of the drawn file name after `prefix`; fails the test when the name does not start with it.
fn suffix_after(path: &Path, prefix: &str) -> String {
    let name = path.file_name().and_then(|name| name.to_str()).unwrap_or_default().to_string();
    assert!(name.starts_with(prefix), "{name} does not start with {prefix}");
    name[prefix.len()..].to_string()
}

#[test]
fn two_draws_for_the_same_path_differ() {
    let path = Path::new("/a/b/hooks/pre-push");
    assert_ne!(draw_temp_path(path), draw_temp_path(path));
}

#[test]
fn a_thousand_draws_are_all_distinct() {
    let path = Path::new("/a/b/hooks/pre-push");
    let drawn: HashSet<_> = (0..1000).map(|_| draw_temp_path(path)).collect();
    assert_eq!(drawn.len(), 1000);
}

#[test]
fn sits_in_the_same_folder() {
    assert_eq!(draw_temp_path(Path::new("/a/b/hooks/pre-push")).parent(), Some(Path::new("/a/b/hooks")));
}

#[test]
fn name_is_the_file_name_then_sixteen_lowercase_hex_digits() {
    let suffix = suffix_after(&draw_temp_path(Path::new("/a/b/hooks/pre-push")), "pre-push.mortar-tmp-");
    assert_eq!(suffix.len(), 16);
    assert!(suffix.chars().all(|c| matches!(c, '0'..='9' | 'a'..='f')), "{suffix} is not lowercase hex");
}

#[test]
fn name_is_never_the_bare_fixed_temp_name() {
    let drawn = draw_temp_path(Path::new("/a/b/hooks/pre-push"));
    assert_ne!(drawn.file_name(), Some("pre-push.mortar-tmp".as_ref()));
}

#[test]
fn works_for_guard_json() {
    let drawn = draw_temp_path(Path::new("/repo/.git/mortar/guard.json"));
    assert_eq!(drawn.parent(), Some(Path::new("/repo/.git/mortar")));
    let suffix = suffix_after(&drawn, "guard.json.mortar-tmp-");
    assert_eq!(suffix.len(), 16);
    assert!(suffix.chars().all(|c| matches!(c, '0'..='9' | 'a'..='f')), "{suffix} is not lowercase hex");
}
