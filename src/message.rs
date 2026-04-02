use std::path::PathBuf;

use iced::widget::text_editor;

use crate::state::TreeNode;
use crate::state::{HoverTarget, SettingsCategory, SettingsNavTarget};

/// Draft-only updates in the settings panel (applied on Save).
#[derive(Debug, Clone)]
pub enum SettingsPreferenceChanged {
    AutoSaveDebounceMs(u32),
    ShowStatusHoverHints(bool),
    PreviewMaxWidth(u32),
    PreviewBaseTextSize(u16),
    PreviewPaddingVertical(u16),
    PreviewPaddingHorizontal(u16),
    EditorFontSize(u16),
    EditorPaddingHorizontal(u16),
    EditorPaddingVertical(u16),
}

#[derive(Debug, Clone)]
pub enum Message {
    SettingsPressed,
    SettingsCategorySelected(SettingsCategory),
    SettingsDraftThemeSelected(crate::persist::ThemeMode),
    SettingsPreferenceChanged(SettingsPreferenceChanged),
    SaveSettings,
    UnsavedSettingsSave { target: SettingsNavTarget },
    UnsavedSettingsDiscard { target: SettingsNavTarget },
    UnsavedSettingsClose,

    OpenFolderPressed,
    OpenFilePressed,
    FolderChosen(Option<PathBuf>),
    FileChosen(Option<PathBuf>),

    FolderScanDone(Result<(TreeNode, PathBuf), String>),
    InitialFileLoaded(PathBuf, Result<(String, Option<std::time::SystemTime>), String>),
    FileOpenResult(PathBuf, Result<(String, Option<std::time::SystemTime>), String>),

    /// Sidebar folder row: select category, toggle expanded; does not change the open document.
    FolderRowPressed(PathBuf),
    FileSelected(PathBuf),

    ModeViewer,
    ModeEditor,
    ToggleMode,

    EditorAction(text_editor::Action),
    SaveTick,
    SaveCompleted(Result<(), String>),

    FsChange(PathBuf),
    WatcherInitFailed(String),
    TreeRescanTick,
    DocumentReloadCompleted(
        PathBuf,
        Result<(String, Option<std::time::SystemTime>), String>,
    ),

    NewMarkdownFilePressed,
    NewFolderPressed,
    CreateDialogInputChanged(String),
    ConfirmCreateDialog,
    CancelCreateDialog,
    CreateCompleted(Result<CreateResult, String>),

    ConflictReloadFromDisk,
    ConflictKeepLocal,

    LinkClicked(iced::widget::markdown::Uri),

    /// Relative vertical scroll position in the markdown preview (0..=1); `None` = no update.
    PreviewScrolledRel(Option<f32>),

    /// Wheel on the editor overlay: scroll the outer editor [`scrollable`] (see `content.rs`).
    EditorOuterScroll(iced::mouse::ScrollDelta),
    /// Relative vertical scroll for the editor pane (0..=1); `None` = no update.
    EditorScrolledRel(Option<f32>),

    /// User/system asked to close this window; we flush the document then snapshot mode for persist.
    WindowCloseRequested(iced::window::Id),
    /// Final persist after close: `size` is only set when not fullscreen (avoid bogus teardown resizes).
    CloseWindowGeometry {
        fullscreen: bool,
        size: Option<iced::Size>,
    },
    PaneResized(iced::widget::pane_grid::ResizeEvent),

    HoverEnter(HoverTarget),
    HoverLeave(HoverTarget),

    /// Inner size after a resize; mode is resolved asynchronously.
    WindowResized(iced::window::Id, iced::Size),
    /// Persisted geometry: when fullscreen, do not treat `size` as the windowed size.
    WindowGeometryPersist {
        size: iced::Size,
        fullscreen: bool,
    },
}

#[derive(Debug, Clone)]
pub struct CreateResult {
    pub path: PathBuf,
    pub is_file: bool,
}
