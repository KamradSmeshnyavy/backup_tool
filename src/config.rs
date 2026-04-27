use crate::AppError;
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub source_dir: PathBuf,
    pub dest_dir: PathBuf,
    pub recipient_public_key: PathBuf,
    // pub log_level: Option<String>,
    pub max_log_size_mb: Option<u64>,
    pub log_file: Option<PathBuf>,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Config, AppError> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| AppError::Config(format!("Cannot read config file: {}", e)))?;
        let cfg: Config = toml::from_str(&content)
            .map_err(|e| AppError::Config(format!("Invalid TOML: {}", e)))?;
        // Validate required fields
        if !cfg.source_dir.exists() {
            return Err(AppError::Config(format!(
                "Source directory does not exist: {:?}",
                cfg.source_dir
            )));
        }
        Ok(cfg)
    }
}
