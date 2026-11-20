//! The files a sparse folder holds outside its paths and the root files (check 1).

/// `files` are the folder's files, relative, `/`-separated. Returns, in order, each file that is neither one of
/// `root_files` nor inside one of `paths` (a folder: the file starts with `<path>/`).
pub fn sparse_strays(files: &[String], paths: &[&str], root_files: &[&str]) -> Vec<String> {
    files
        .iter()
        .filter(|file| {
            !root_files.contains(&file.as_str()) && !paths.iter().any(|path| file.starts_with(&format!("{path}/")))
        })
        .cloned()
        .collect()
}
