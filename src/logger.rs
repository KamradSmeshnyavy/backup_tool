use crate::AppError;
use chrono::Local;
// use log::LevelFilter;
use std::fs;
use std::path::Path;

pub fn init_logger(log_path: &Path, max_size_mb: u64) -> Result<(), AppError> {
    // Ротация при превышении размера
    if let Ok(meta) = fs::metadata(log_path) {
        if meta.len() > max_size_mb * 1_048_576 {
            rotate_logs(log_path, 5)?;
        }
    }

    // Используем env_logger, который пишет в stderr по умолчанию,
    // но можно настроить запись в файл через Builder.
    let file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path)
        .map_err(|e| AppError::Logger(format!("Cannot open log file: {}", e)))?;

    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Info)
        .format_timestamp_secs() // будем использовать секунды
        .target(env_logger::Target::Pipe(Box::new(file)))
        .try_init()
        .map_err(|e| AppError::Logger(format!("Logger init failed: {}", e)))?;

    Ok(())
}

fn rotate_logs(log_path: &Path, max_files: usize) -> Result<(), AppError> {
    let parent = log_path.parent().unwrap_or(Path::new("."));
    let stem = log_path
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    let timestamp = Local::now().format("%Y%m%d_%H%M%S");
    let new_name = format!("{}_{}.log", stem, timestamp);
    let new_path = parent.join(&new_name);
    fs::rename(log_path, &new_path)?;

    // Удаление старых логов, если их больше max_files
    let mut log_files: Vec<_> = std::fs::read_dir(parent)?
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_name().to_string_lossy().starts_with(&stem)
                && e.file_name().to_string_lossy().ends_with(".log")
        })
        .collect();
    log_files.sort_by_key(|e| e.metadata().and_then(|m| m.created()).ok());

    if log_files.len() > max_files {
        for entry in log_files.iter().take(log_files.len() - max_files) {
            let _ = fs::remove_file(entry.path());
        }
    }
    Ok(())
}
