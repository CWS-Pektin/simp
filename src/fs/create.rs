use std::path::Path;

pub async fn create_markdown_file(path: &Path) -> anyhow::Result<()> {
    let path = path.to_path_buf();
    tokio::task::spawn_blocking(move || {
        if let Some(p) = path.parent() {
            std::fs::create_dir_all(p)?;
        }
        if path.exists() {
            anyhow::bail!("file already exists");
        }
        std::fs::write(&path, b"")?;
        Ok::<_, anyhow::Error>(())
    })
    .await?
}

pub async fn create_folder(path: &Path) -> anyhow::Result<()> {
    let path = path.to_path_buf();
    tokio::task::spawn_blocking(move || {
        if path.exists() {
            anyhow::bail!("folder already exists");
        }
        std::fs::create_dir_all(&path)?;
        Ok::<_, anyhow::Error>(())
    })
    .await?
}
