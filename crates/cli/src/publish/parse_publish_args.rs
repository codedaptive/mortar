//! `mr publish`'s words: --no-push at most once, nothing else.

/// Reads `rest`: whether this publish pushes. With no words it pushes (true); --no-push makes it not push (false). The first failure in line order
/// is the reason returned: a second --no-push, or any other word, named.
pub fn parse_publish_args(rest: &[String]) -> Result<bool, String> {
    let mut push = true;
    for word in rest {
        match word.as_str() {
            "--no-push" if !push => return Err("--no-push is given twice".to_string()),
            "--no-push" => push = false,
            other => return Err(format!("unexpected word {other:?}")),
        }
    }
    Ok(push)
}
