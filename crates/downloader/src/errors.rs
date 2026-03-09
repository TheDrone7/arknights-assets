use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use tokio::fs::{self, OpenOptions};
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc;

pub struct ErrorLogger {
    log_path: PathBuf,
    sender: mpsc::Sender<String>,
}

impl ErrorLogger {
    pub async fn init(base_path: &Path) -> Result<Self> {
        let log_path = base_path.join("errors.log");
        fs::write(&log_path, b"")
            .await
            .with_context(|| "Failed to clear the errors log.")?;

        let (tx, mut rx) = mpsc::channel::<String>(100);
        let writer_path = log_path.clone();

        tokio::spawn(async move {
            if let Ok(mut file) = OpenOptions::new().append(true).open(&writer_path).await {
                while let Some(msg) = rx.recv().await {
                    let _ = file
                        .write_all(format!("{}\n", msg).as_bytes())
                        .await
                        .with_context(|| "Failed to write error to log file.");
                }
            }
        });

        Ok(Self {
            log_path,
            sender: tx,
        })
    }

    pub async fn log_error(&self, message: String) {
        let _ = self
            .sender
            .send(message)
            .await
            .with_context(|| "Failed to log error");
    }

    pub async fn has_errors(&self) -> bool {
        fs::metadata(&self.log_path)
            .await
            .map(|m| m.len() > 0)
            .unwrap_or(false)
    }

    pub fn path(&self) -> &PathBuf {
        &self.log_path
    }
}
