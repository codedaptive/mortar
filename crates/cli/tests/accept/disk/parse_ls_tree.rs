//! Reads the output of `git ls-tree -r -l -z HEAD`.

use super::TreeFile;

/// One TreeFile per NUL-ended record `<mode> <type> <oid> <size>\t<path>`, in order. The size is right-aligned with
/// spaces, and `-` (a submodule commit) gives None. An empty piece and a record not of that shape are skipped.
pub fn parse_ls_tree(output: &str) -> Vec<TreeFile> {
    output
        .split('\0')
        .filter_map(|record| {
            let (meta, path) = record.split_once('\t')?;
            let fields: Vec<&str> = meta.split_whitespace().collect();
            let [mode, _kind, _oid, size] = fields[..] else {
                return None;
            };
            let size = match size {
                "-" => None,
                bytes => Some(bytes.parse().ok()?),
            };
            Some(TreeFile { mode: mode.to_string(), path: path.to_string(), size })
        })
        .collect()
}
