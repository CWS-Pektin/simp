use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum RootContext {
    Folder { root_path: PathBuf },
    SingleFile { file_path: PathBuf },
}

impl RootContext {
    #[allow(dead_code)]
    pub fn root_display_path(&self) -> &std::path::Path {
        match self {
            RootContext::Folder { root_path } => root_path.as_path(),
            RootContext::SingleFile { file_path } => file_path.parent().unwrap_or(file_path.as_path()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Selection {
    Category(PathBuf),
    File(PathBuf),
}
