//! ancestor_refusal: no link answers None; a link on either side is refused naming it, the parent's first.

use std::path::Path;

use super::{ancestor_refusal, BuildDirClone};

struct Row {
    name: &'static str,
    parent_link: Option<&'static str>,
    folder_link: Option<&'static str>,
    want: Option<BuildDirClone>,
}

fn refused(why: &str) -> Option<BuildDirClone> {
    Some(BuildDirClone::Refused { dir: "build/out".to_string(), why: why.to_string() })
}

#[test]
fn ancestor_refusal_names_the_symlinked_ancestor() {
    let rows = [
        Row { name: "no symlinked ancestor on either side", parent_link: None, folder_link: None, want: None },
        Row {
            name: "the parent's ancestor is a symlink",
            parent_link: Some("build"),
            folder_link: None,
            want: refused(
                r#"the parent's "build", an ancestor of build dir "build/out", is a symlink; nothing copied"#,
            ),
        },
        Row {
            name: "the folder's ancestor is a symlink",
            parent_link: None,
            folder_link: Some("build"),
            want: refused(
                r#"the folder's "build", an ancestor of build dir "build/out", is a symlink; nothing copied"#,
            ),
        },
        Row {
            name: "both sides: the parent's is named",
            parent_link: Some("build"),
            folder_link: Some("build"),
            want: refused(
                r#"the parent's "build", an ancestor of build dir "build/out", is a symlink; nothing copied"#,
            ),
        },
    ];
    for row in rows {
        let got = ancestor_refusal("build/out", row.parent_link.map(Path::new), row.folder_link.map(Path::new));
        assert_eq!(got, row.want, "{}", row.name);
    }
}
