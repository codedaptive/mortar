//! Why the config cannot be read, decoded or written.
//! A bare type (S-7); decode_config_tests exercises each decode reason.

/// `reason` names the key, value or byte that is wrong; from read_config and write_config it names the file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigError {
    pub reason: String,
}
