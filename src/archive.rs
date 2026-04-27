use crate::AppError;
use flate2::write::GzEncoder;
use flate2::Compression;
use std::path::{Path, PathBuf};

pub fn create_archive(source_dir: &Path, files: &[PathBuf]) -> Result<Vec<u8>, AppError> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    {
        let mut tar = tar::Builder::new(&mut encoder);
        for relative in files {
            let full_path = source_dir.join(relative);
            tar.append_path_with_name(&full_path, relative)
                .map_err(|e| AppError::Archive(format!("Tar append error: {}", e)))?;
        }
        tar.finish()
            .map_err(|e| AppError::Archive(format!("Tar finish error: {}", e)))?;
    }
    let compressed = encoder
        .finish()
        .map_err(|e| AppError::Archive(format!("Gz finish error: {}", e)))?;
    Ok(compressed)
}
