//! Free disk as `df` reports it: runs `df -P -k <path>`; read_df_available reads the output.
//! No unit test: it runs a real program against a real volume. Covered by Nagatha's integration
//! group: free_bytes on the test's temp folder answers a positive figure.

use std::path::Path;
use std::process::Command;

use super::{read_df_available, FreeDisk};

/// The free-disk answer that asks the OS through `df`.
pub struct DfFreeDisk;

impl FreeDisk for DfFreeDisk {
    /// Err when df cannot be run, or exits non-zero (with its stderr), or prints output that
    /// read_df_available refuses.
    fn free_bytes(&self, path: &Path) -> Result<u64, String> {
        let out = Command::new("df")
            .arg("-P")
            .arg("-k")
            .arg(path)
            .output()
            .map_err(|e| format!("cannot run df: {e}"))?;
        if !out.status.success() {
            return Err(format!(
                "df -P -k {} exited with {}: {}",
                path.display(),
                out.status,
                String::from_utf8_lossy(&out.stderr).trim()
            ));
        }
        read_df_available(&out.stdout)
    }
}
