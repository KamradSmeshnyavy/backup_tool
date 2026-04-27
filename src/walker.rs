use crate::AppError;
use log::warn;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub fn list_files(source_dir: &Path) -> Result<Vec<PathBuf>, AppError> {
    let mut files = Vec::new();
    for entry in WalkDir::new(source_dir)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        if path.is_symlink() {
            warn!("Skipping symlink: {:?}", path);
            continue;
        }
        // Проверка доступа: если метаданные недоступны, пропускаем с предупреждением
        let _meta = match path.metadata() {
            Ok(m) => m,
            Err(e) => {
                warn!("Cannot read metadata for {:?}: {}", path, e);
                continue;
            }
        };
        // (дополнительная проверка прав, если необходимо)
        let relative = path
            .strip_prefix(source_dir)
            .map_err(|e| AppError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
        files.push(relative.to_path_buf());
    }
    Ok(files)
}
