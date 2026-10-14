use std::path::{Path, PathBuf};

pub fn executable_dir() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(Path::to_path_buf))
        .unwrap_or_else(|| PathBuf::from("."))
}

pub fn join(base: &Path, name: &str) -> PathBuf {
    base.join(name)
}

pub fn file_name(path: &Path) -> Option<String> {
    path.file_name().map(|name| name.to_string_lossy().into_owned())
}

pub fn extension(path: &Path) -> Option<String> {
    path.extension().map(|ext| ext.to_string_lossy().into_owned())
}

pub fn file_pure_name(path: &Path) -> String {
    path.file_stem()
        .map(|stem| stem.to_string_lossy().into_owned())
        .unwrap_or_default()
}
