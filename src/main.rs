use clap::{Parser, Subcommand};
use log::info;
use std::path::PathBuf;

mod archive;
mod config;
mod crypto;
mod error;
mod logger;
mod walker;

use config::Config;
use crypto::{decrypt_backup, encrypt_backup, load_public_key, load_secret_key, EncryptedKey};
use error::AppError;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Создать резервную копию
    Backup {
        #[arg(short, long)]
        config: PathBuf,
    },
    /// Восстановить из резервной копии
    Restore {
        #[arg(short, long)]
        config: PathBuf,
        #[arg(short = 'k', long)]
        secret_key: PathBuf,
        #[arg(short = 'e', long)]
        enc_key: PathBuf,
        #[arg(short, long)]
        input: PathBuf,
        #[arg(short, long)]
        output: PathBuf,
    },
}

fn main() -> Result<(), AppError> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Backup { config } => {
            let cfg = Config::from_file(config.to_str().unwrap())?;
            let log_file = cfg
                .log_file
                .clone()
                .unwrap_or_else(|| PathBuf::from("backup_tool.log"));
            let max_log_mb = cfg.max_log_size_mb.unwrap_or(5);
            logger::init_logger(&log_file, max_log_mb)?;

            info!("Starting backup of {:?}", cfg.source_dir);

            let files = walker::list_files(&cfg.source_dir)?;
            info!("Found {} files", files.len());

            let archive_data = archive::create_archive(&cfg.source_dir, &files)?;
            info!("Archive size: {:.2} MB", archive_data.len() as f64 / 1e6);

            let recipient_pk = load_public_key(&cfg.recipient_public_key)?;
            let (encrypted_archive, envelope) = encrypt_backup(&archive_data, recipient_pk)?;

            let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
            let dest = cfg
                .dest_dir
                .join(format!("backup_{}.tar.gz.enc", timestamp));
            std::fs::write(&dest, &encrypted_archive)?;
            info!("Encrypted archive saved to {:?}", dest);

            let key_file = cfg.dest_dir.join(format!("backup_{}.key", timestamp));
            let key_bytes = bincode::serialize(&envelope)
                .map_err(|e| AppError::Config(format!("Serialization error: {}", e)))?;
            std::fs::write(&key_file, key_bytes)?;
            info!("Key envelope saved to {:?}", key_file);

            info!("Backup completed successfully.");
        }
        Commands::Restore {
            config,
            secret_key,
            enc_key,
            input,
            output,
        } => {
            let cfg = Config::from_file(config.to_str().unwrap())?;
            let log_file = cfg
                .log_file
                .clone()
                .unwrap_or_else(|| PathBuf::from("backup_tool.log"));
            let max_log_mb = cfg.max_log_size_mb.unwrap_or(5);
            logger::init_logger(&log_file, max_log_mb)?;

            info!("Starting restore to {:?}", output);

            let secret = load_secret_key(&secret_key)?;
            let envelope_bytes = std::fs::read(&enc_key)?;
            let envelope: EncryptedKey = bincode::deserialize(&envelope_bytes)
                .map_err(|e| AppError::Config(format!("Deserialization error: {}", e)))?;
            let encrypted_data = std::fs::read(&input)?;
            let plain_archive = decrypt_backup(&encrypted_data, &secret, &envelope)?;

            let mut decoder = flate2::read::GzDecoder::new(plain_archive.as_slice());
            let mut archive = tar::Archive::new(&mut decoder);
            archive
                .unpack(&output)
                .map_err(|e| AppError::Archive(format!("Unpack error: {}", e)))?;

            info!("Restore completed successfully.");
        }
    }

    Ok(())
}
