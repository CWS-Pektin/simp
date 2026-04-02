use std::path::Path;

pub async fn save_atomic(path: &Path, contents: &str) -> anyhow::Result<()> {
    let path = path.to_path_buf();
    let contents = contents.to_string();
    tokio::task::spawn_blocking(move || {
        let parent = path.parent().ok_or_else(|| anyhow::anyhow!("no parent"))?;
        std::fs::create_dir_all(parent)?;
        let tmp = path.with_extension("md.tmp");
        std::fs::write(&tmp, contents.as_bytes())?;
        #[cfg(unix)]
        {
            std::fs::rename(&tmp, &path)?;
        }
        #[cfg(windows)]
        {
            if path.exists() {
                std::fs::remove_file(&path)?;
            }
            std::fs::rename(&tmp, &path)?;
        }
        Ok::<_, anyhow::Error>(())
    })
    .await?
}
