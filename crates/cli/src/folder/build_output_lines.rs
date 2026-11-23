//! build_output_lines: the lines `mr folder request` adds for each build dir it did not share.

use folders::BuildDirClone;

/// One line per build dir that was not shared, in the order given; a shared dir adds no line.
pub fn build_output_lines(clones: &[BuildDirClone]) -> Vec<String> {
    clones
        .iter()
        .filter_map(|clone| match clone {
            BuildDirClone::Shared { .. } => None,
            BuildDirClone::Absent { dir } => Some(format!("build dir {dir} not shared: the zone folder has none")),
            BuildDirClone::Kept { dir } => {
                Some(format!("build dir {dir} not shared: the folder already has it, kept as it is"))
            }
            BuildDirClone::Refused { dir, why } => Some(format!("build dir {dir} not shared: {why}")),
        })
        .collect()
}
