use std::{fs, os::unix::fs::PermissionsExt, path::Path};

pub fn write_hook(common: &Path, name: &str, body: &[u8]) {
    let hooks = common.join("hooks");
    fs::create_dir_all(&hooks).unwrap();
    let path = hooks.join(name);
    fs::write(&path, body).unwrap();
    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).unwrap();
}
