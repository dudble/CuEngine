use std::fs;
use std::io;
use std::path::{Path, PathBuf};

pub fn read_to_string(path: &Path) -> io::Result<String> {
    fs::read_to_string(path)
}

pub fn exists(path: &Path) -> bool {
    path.exists()
}

pub fn list_files(directory: &Path) -> io::Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        if entry.file_type()?.is_file() {
            files.push(entry.path());
        }
    }
    Ok(files)
}
