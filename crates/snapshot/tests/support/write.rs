use std::{fs, path::Path};

pub fn write(folder: &Path, name: &str, bytes: &[u8]) {
    let path = folder.join(name);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, bytes).unwrap();
}
