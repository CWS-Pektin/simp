use std::path::PathBuf;

use crate::state::SettingsNavTarget;

#[derive(Debug, Clone)]
pub enum DialogState {
    NewMarkdownFile { parent: PathBuf, input: String },
    NewFolder { parent: PathBuf, input: String },
    /// Unsaved settings: choose Save, Discard, or Close (stay and keep editing).
    UnsavedSettings {
        target: SettingsNavTarget,
    },
}
