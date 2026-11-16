//! The report line saying whether the integration folder was moved.

use std::path::Path;

use gitio::Oid;

/// "moved the integration folder <folder> to <new>" when a folder moved, else "no integration folder to move".
pub fn integration_folder_line(moved: Option<&Path>, new: &Oid) -> String {
    match moved {
        Some(folder) => format!("moved the integration folder {} to {new}", folder.display()),
        None => "no integration folder to move".to_string(),
    }
}
