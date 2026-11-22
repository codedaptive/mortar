//! The report's commit line. Pure: read_commit.rs passes it what git said.

/// `commit <id>`, with ` with local changes` when the tree differs from it; when only the local changes were not
/// read, the id and why; when the commit was not read, why, whatever the local changes said.
pub fn commit_line(head: &Result<String, String>, changed: &Result<bool, String>) -> String {
    match (head, changed) {
        (Ok(id), Ok(false)) => format!("commit {id}"),
        (Ok(id), Ok(true)) => format!("commit {id} with local changes"),
        (Ok(id), Err(why)) => format!("commit {id} (whether it has local changes was not read: {why})"),
        (Err(why), _) => format!("commit not read: {why}"),
    }
}
