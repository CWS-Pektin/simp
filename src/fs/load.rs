use std::path::Path;
use std::time::SystemTime;

pub async fn load_file(path: &Path) -> anyhow::Result<(String, Option<SystemTime>)> {
    let path = path.to_path_buf();
    tokio::task::spawn_blocking(move || {
        let meta = std::fs::metadata(&path)?;
        let mtime = meta.modified().ok();
        let text = std::fs::read_to_string(&path)?;
        Ok::<_, anyhow::Error>((text, mtime))
    })
    .await?
}
