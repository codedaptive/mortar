//! The changed lines of a re-review's range-diff, each with the section it is in (check 9).

/// `lines` are `mr rereview`'s, git's range-diff of the two squashes. Its body is a diff of the two patches, each
/// line indented four spaces and then marked ` `, `+` or `-`; a section starts at an outer hunk header
/// (`    @@ <section>`, naming the section the hunk starts in) or at a patch's own `## <section> ##` line, which git
/// writes after the mark with a space before it, where the section is `Metadata`, `Commit message` or a file path
/// (git may add ` (new)`, ` (deleted)` or `: <context>`, which are dropped). Returns, in order, each line marked
/// `+` or `-` as (its section, the line without its four-space indent: the outer mark, then the patch line, e.g.
/// `-+r1 line 2`); "" is the section of a change before any section starts.
pub fn rereview_changes(lines: &[String]) -> Vec<(String, String)> {
    let mut section = String::new();
    let mut changes = Vec::new();
    for line in lines {
        let Some(body) = line.strip_prefix("    ") else {
            continue;
        };
        if let Some(header) = body.strip_prefix("@@ ") {
            section = section_name(header);
            continue;
        }
        let mut chars = body.chars();
        let mark = chars.next();
        if let Some(name) = chars.as_str().trim_start().strip_prefix("## ").and_then(|rest| rest.strip_suffix(" ##")) {
            section = section_name(name);
        }
        if matches!(mark, Some('+' | '-')) {
            changes.push((section.clone(), body.to_string()));
        }
    }
    changes
}

/// A section's name without git's ` (new)`-style note or its `: <context>`.
fn section_name(text: &str) -> String {
    let text = text.split_once(": ").map_or(text, |(name, _)| name);
    text.split_once(" (").map_or(text, |(name, _)| name).trim().to_string()
}
