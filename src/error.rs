use std::fmt;

#[derive(Debug)]
pub enum AppError {
    Io(std::io::Error),
    Config(String),
    Crypto(String),
    Archive(String),
    Logger(String),
}

impl std::error::Error for AppError {}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Io(e) => write!(f, "IO error: {}", e),
            AppError::Config(msg) => write!(f, "Config error: {}", msg),
            AppError::Crypto(msg) => write!(f, "Crypto error: {}", msg),
            AppError::Archive(msg) => write!(f, "Archive error: {}", msg),
            AppError::Logger(msg) => write!(f, "Logger error: {}", msg),
        }
    }
}

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        AppError::Io(e)
    }
}
