use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::state::ContentMode;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ThemeMode {
    #[default]
    Light,
    Dark,
}

impl std::fmt::Display for ThemeMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ThemeMode::Light => write!(f, "Light"),
            ThemeMode::Dark => write!(f, "Dark"),
        }
    }
}

// ── User preferences (nested JSON; serde defaults keep old configs valid) ─────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GeneralPreferences {
    /// Milliseconds of idle time after an edit before auto-save is attempted.
    #[serde(default = "default_auto_save_debounce_ms")]
    pub auto_save_debounce_ms: u32,
    /// When false, the status bar shows a placeholder instead of hover help.
    #[serde(default = "default_true")]
    pub show_status_hover_hints: bool,
}

fn default_auto_save_debounce_ms() -> u32 {
    250
}

fn default_true() -> bool {
    true
}

impl Default for GeneralPreferences {
    fn default() -> Self {
        Self {
            auto_save_debounce_ms: default_auto_save_debounce_ms(),
            show_status_hover_hints: default_true(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PreviewPreferences {
    #[serde(default = "default_preview_max_width")]
    pub max_content_width: u32,
    /// Markdown preview body text size in logical pixels (same unit as CSS `px` / Iced `Pixels`).
    #[serde(default = "default_preview_base_size")]
    pub base_text_size: u16,
    #[serde(default = "default_preview_pad_v")]
    pub padding_vertical: u16,
    #[serde(default = "default_preview_pad_h")]
    pub padding_horizontal: u16,
}

fn default_preview_max_width() -> u32 {
    820
}

fn default_preview_base_size() -> u16 {
    16
}

fn default_preview_pad_v() -> u16 {
    32
}

fn default_preview_pad_h() -> u16 {
    48
}

impl Default for PreviewPreferences {
    fn default() -> Self {
        Self {
            max_content_width: default_preview_max_width(),
            base_text_size: default_preview_base_size(),
            padding_vertical: default_preview_pad_v(),
            padding_horizontal: default_preview_pad_h(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditorPreferences {
    /// Source editor monospace size in logical pixels (same unit as CSS `px` / Iced `Pixels`).
    #[serde(default = "default_editor_font")]
    pub font_size: u16,
    #[serde(default = "default_editor_pad_h")]
    pub padding_horizontal: u16,
    #[serde(default = "default_editor_pad_v")]
    pub padding_vertical: u16,
}

fn default_editor_font() -> u16 {
    14
}

fn default_editor_pad_h() -> u16 {
    20
}

fn default_editor_pad_v() -> u16 {
    16
}

impl Default for EditorPreferences {
    fn default() -> Self {
        Self {
            font_size: default_editor_font(),
            padding_horizontal: default_editor_pad_h(),
            padding_vertical: default_editor_pad_v(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct UserPreferences {
    #[serde(default)]
    pub general: GeneralPreferences,
    #[serde(default)]
    pub preview: PreviewPreferences,
    #[serde(default)]
    pub editor: EditorPreferences,
}

impl UserPreferences {
    /// Clamp fields to supported ranges after load or UI input.
    pub fn sanitized(mut self) -> Self {
        self.general.auto_save_debounce_ms = self
            .general
            .auto_save_debounce_ms
            .clamp(250, 2000);
        self.preview.max_content_width = self.preview.max_content_width.clamp(480, 1400);
        self.preview.base_text_size = self.preview.base_text_size.clamp(12, 22);
        self.preview.padding_vertical = self.preview.padding_vertical.clamp(16, 64);
        self.preview.padding_horizontal = self.preview.padding_horizontal.clamp(16, 80);
        self.editor.font_size = self.editor.font_size.clamp(11, 22);
        self.editor.padding_horizontal = self.editor.padding_horizontal.clamp(8, 40);
        self.editor.padding_vertical = self.editor.padding_vertical.clamp(8, 40);
        self
    }
}

/// The subset of app state that survives across restarts.
/// Folder workspace (root, tree UI, last file, theme) is persisted; ad-hoc "Open File"-only sessions are not.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PersistState {
    /// Last opened root folder (None means nothing to restore).
    pub root_folder: Option<PathBuf>,
    /// Which category nodes were expanded in the sidebar.
    pub expanded: Vec<PathBuf>,
    /// Which category was selected when no file tab is restored.
    pub selected_category: Option<PathBuf>,
    /// Last-open Markdown file under `root_folder` (folder workspace only).
    #[serde(default)]
    pub open_file: Option<PathBuf>,
    /// Theme mode (light or dark).
    pub theme_mode: ThemeMode,
    /// Default preview vs editor for files not listed in `file_content_modes`.
    #[serde(default)]
    pub content_mode: ContentMode,
    /// Per-file editor vs preview (folder workspace; keys are canonical paths when possible).
    #[serde(default)]
    pub file_content_modes: HashMap<PathBuf, ContentMode>,
    /// Per-file vertical scroll in preview as a relative offset in 0..=1 (folder workspace).
    #[serde(default)]
    pub preview_scroll_rel_y: HashMap<PathBuf, f32>,
    /// Per-file vertical scroll in the editor pane (same semantics as preview).
    #[serde(default)]
    pub editor_scroll_rel_y: HashMap<PathBuf, f32>,

    /// Last windowed inner width in logical pixels (`None` = use default size on first run).
    #[serde(default)]
    pub window_width: Option<f32>,
    /// Last windowed inner height in logical pixels.
    #[serde(default)]
    pub window_height: Option<f32>,
    /// Whether the window was fullscreen when last saved.
    #[serde(default)]
    pub window_fullscreen: bool,

    #[serde(default)]
    pub preferences: UserPreferences,
}

impl PersistState {
    pub fn expanded_set(&self) -> HashSet<PathBuf> {
        self.expanded.iter().cloned().collect()
    }
}

fn config_path() -> Option<PathBuf> {
    std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .map(|home| PathBuf::from(home).join(".markdownview.json"))
}

pub fn load() -> PersistState {
    let Some(path) = config_path() else {
        return Default::default();
    };
    let Ok(data) = std::fs::read_to_string(&path) else {
        return Default::default();
    };
    let mut state: PersistState = serde_json::from_str(&data).unwrap_or_default();
    state.preferences = state.preferences.clone().sanitized();
    state
}

pub fn save(state: &PersistState) {
    let Some(path) = config_path() else { return };
    if let Ok(json) = serde_json::to_string_pretty(state) {
        let _ = std::fs::write(&path, json);
    }
}
