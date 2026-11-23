//! build_output_lines: each BuildDirClone in the cli's words, shared dirs adding no line.

use folders::BuildDirClone;

use super::build_output_lines;

/// A case's name, the clones given, and the lines expected.
type Case = (&'static str, Vec<BuildDirClone>, Vec<&'static str>);

fn dir(name: &str) -> String {
    name.to_string()
}

#[test]
fn build_output_lines_cases() {
    let cases: Vec<Case> = vec![
        ("no clones", vec![], vec![]),
        ("shared only", vec![BuildDirClone::Shared { dir: dir("target") }], vec![]),
        (
            "absent",
            vec![BuildDirClone::Absent { dir: dir("build") }],
            vec!["build dir build not shared: the zone folder has none"],
        ),
        (
            "kept",
            vec![BuildDirClone::Kept { dir: dir("target") }],
            vec!["build dir target not shared: the folder already has it, kept as it is"],
        ),
        (
            "refused",
            vec![BuildDirClone::Refused { dir: dir("target"), why: dir("it is a \"symlink\"") }],
            vec!["build dir target not shared: it is a \"symlink\""],
        ),
        (
            "mix in order, shared adds none",
            vec![
                BuildDirClone::Shared { dir: dir("target") },
                BuildDirClone::Absent { dir: dir("build") },
                BuildDirClone::Shared { dir: dir("out") },
                BuildDirClone::Kept { dir: dir("dist") },
                BuildDirClone::Refused { dir: dir("gen"), why: dir("no clonefile here") },
            ],
            vec![
                "build dir build not shared: the zone folder has none",
                "build dir dist not shared: the folder already has it, kept as it is",
                "build dir gen not shared: no clonefile here",
            ],
        ),
    ];
    for (name, clones, expected) in cases {
        assert_eq!(build_output_lines(&clones), expected, "case {name}");
    }
}
