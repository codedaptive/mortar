//! build_output_json: each BuildDirClone as one encoded JSON object, in order, with the why escaped.

use folders::BuildDirClone;

use super::build_output_json;

fn dir(name: &str) -> String {
    name.to_string()
}

#[test]
fn build_output_json_cases() {
    let cases: Vec<(&str, Vec<BuildDirClone>, Vec<&str>)> = vec![
        ("none", vec![], vec![]),
        ("shared", vec![BuildDirClone::Shared { dir: dir("target") }], vec![r#"{"dir":"target","outcome":"shared"}"#]),
        ("absent", vec![BuildDirClone::Absent { dir: dir("build") }], vec![r#"{"dir":"build","outcome":"absent"}"#]),
        ("kept", vec![BuildDirClone::Kept { dir: dir("target") }], vec![r#"{"dir":"target","outcome":"kept"}"#]),
        (
            "refused with a quote and a backslash",
            vec![BuildDirClone::Refused { dir: dir("target"), why: dir(r#"the dir "target" is at C:\out"#) }],
            vec![r#"{"dir":"target","outcome":"refused","why":"the dir \"target\" is at C:\\out"}"#],
        ),
        (
            "mix in order",
            vec![
                BuildDirClone::Shared { dir: dir("target") },
                BuildDirClone::Absent { dir: dir("build") },
                BuildDirClone::Kept { dir: dir("dist") },
            ],
            vec![
                r#"{"dir":"target","outcome":"shared"}"#,
                r#"{"dir":"build","outcome":"absent"}"#,
                r#"{"dir":"dist","outcome":"kept"}"#,
            ],
        ),
    ];
    for (name, clones, expected) in cases {
        assert_eq!(build_output_json(&clones), expected, "case {name}");
    }
}
