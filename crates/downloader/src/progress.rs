use anyhow::Result;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::fs;

pub struct ProgressTracker {
    progress_file: PathBuf,
    pub completed: HashMap<String, String>,
}

impl ProgressTracker {
    pub async fn load(base_path: &Path) -> Result<Self> {
        let progress_file = base_path.join("downloaded.json");
        let mut completed = HashMap::new();

        if progress_file.exists() {
            let data = fs::read_to_string(&progress_file).await?;
            if let Ok(parsed) = serde_json::from_str(&data) {
                completed = parsed;
            }
        }

        Ok(Self {
            progress_file,
            completed,
        })
    }

    pub fn is_up_to_date(&self, file_name: &str, md5: &str) -> bool {
        self.completed.get(file_name).map(|s| s.as_str()) == Some(md5)
    }

    pub async fn mark_completed(&mut self, file_name: String, md5: String) -> Result<()> {
        self.completed.insert(file_name, md5);
        self.save().await
    }

    pub async fn save(&self) -> Result<()> {
        let data = serde_json::to_string(&self.completed)?;
        fs::write(&self.progress_file, data).await?;
        Ok(())
    }
}
