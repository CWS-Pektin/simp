use std::path::PathBuf;

use crate::state::SettingsCategory;

/// Identifies a hovered UI region for the status-bar help strip.
/// Used with matched enter/leave so moving between controls does not clear the wrong tooltip.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HoverTarget {
    SettingsButton,
    OpenFolderButton,
    OpenFileButton,
    EditorPreviewToggle,
    /// Second menu row: current file / selection / workspace path.
    MenuPathContext,
    /// Settings mode hint under the path.
    MenuSettingsHint,
    /// Bottom-right: save state, clean document, errors.
    StatusDocument,
    SettingsSidebarHeader,
    SettingsSidebarCategory(SettingsCategory),
    SettingsThemePicker,
    SettingsAutoSaveDebounce,
    SettingsShowStatusHints,
    SettingsPreviewMaxWidth,
    SettingsPreviewBaseSize,
    SettingsPreviewPaddingV,
    SettingsPreviewPaddingH,
    SettingsEditorFontSize,
    SettingsEditorPaddingH,
    SettingsEditorPaddingV,
    SettingsSave,
    DialogUnsavedSave,
    DialogUnsavedDiscard,
    DialogUnsavedClose,
    DialogCreateSubmit,
    DialogCreateCancel,
    DialogCreateNameInput,
    ContentEmptyHint,
    ContentPreviewPane,
    ContentEditorPane,
    ConflictReloadFromDisk,
    ConflictKeepLocal,
    SidebarFile(PathBuf),
    /// Sidebar folder row: expand/collapse and select target for new file/folder.
    SidebarFolderRow(PathBuf),
    /// Sidebar header: create Markdown file in the selected folder.
    SidebarHeaderNewFile,
    /// Sidebar header: create subfolder in the selected folder.
    SidebarHeaderNewFolder,
    /// Sidebar header showing the project root name.
    SidebarProjectHeader,
    /// Placeholder when no folder is open.
    SidebarEmptyState,
}

impl HoverTarget {
    pub fn help_line(&self) -> String {
        match self {
            HoverTarget::SettingsButton => "Open or close settings.".into(),
            HoverTarget::OpenFolderButton => "Choose a folder to use as the project root.".into(),
            HoverTarget::OpenFileButton => "Open a single Markdown file without a project folder.".into(),
            HoverTarget::EditorPreviewToggle => {
                "Switch between rendered preview and source editor for the open file.".into()
            }
            HoverTarget::MenuPathContext => {
                "Shows the open file path, the selected sidebar item path, or — when neither applies."
                    .into()
            }
            HoverTarget::MenuSettingsHint => {
                "Adjust options in the main area. Use Save to apply changes.".into()
            }
            HoverTarget::StatusDocument => {
                "Save progress, whether the file matches disk, and load or conflict messages."
                    .into()
            }
            HoverTarget::SettingsSidebarHeader => "Settings navigation.".into(),
            HoverTarget::SettingsSidebarCategory(SettingsCategory::General) => {
                "Auto-save timing and status bar behavior.".into()
            }
            HoverTarget::SettingsSidebarCategory(SettingsCategory::Preview) => {
                "Layout and typography for the Markdown preview.".into()
            }
            HoverTarget::SettingsSidebarCategory(SettingsCategory::Editor) => {
                "Font size and padding for the source editor.".into()
            }
            HoverTarget::SettingsSidebarCategory(SettingsCategory::Appearance) => {
                "Theme and appearance options.".into()
            }
            HoverTarget::SettingsThemePicker => {
                "Choose light or dark theme (apply with Save when it differs from the current theme)."
                    .into()
            }
            HoverTarget::SettingsAutoSaveDebounce => {
                "How long to wait after you stop typing before saving the open file.".into()
            }
            HoverTarget::SettingsShowStatusHints => {
                "When on, the status bar shows help for the control under the pointer.".into()
            }
            HoverTarget::SettingsPreviewMaxWidth => {
                "Maximum width of the preview column (content stays centered).".into()
            }
            HoverTarget::SettingsPreviewBaseSize => {
                "Base font size for rendered Markdown in the preview, in logical pixels (px)."
                    .into()
            }
            HoverTarget::SettingsPreviewPaddingV => {
                "Space above and below the preview content inside the pane.".into()
            }
            HoverTarget::SettingsPreviewPaddingH => {
                "Space to the left and right of the preview content.".into()
            }
            HoverTarget::SettingsEditorFontSize => {
                "Monospace font size in the Markdown source editor, in logical pixels (px)."
                    .into()
            }
            HoverTarget::SettingsEditorPaddingH => {
                "Horizontal padding inside the editor around the text.".into()
            }
            HoverTarget::SettingsEditorPaddingV => {
                "Vertical padding inside the editor around the text.".into()
            }
            HoverTarget::SettingsSave => "Save settings to disk.".into(),
            HoverTarget::DialogUnsavedSave => "Save settings and continue.".into(),
            HoverTarget::DialogUnsavedDiscard => "Discard unsaved settings and continue.".into(),
            HoverTarget::DialogUnsavedClose => "Stay in settings and keep editing.".into(),
            HoverTarget::DialogCreateSubmit => "Create with the name above (extension added for files)."
                .into(),
            HoverTarget::DialogCreateCancel => "Close this dialog without creating.".into(),
            HoverTarget::DialogCreateNameInput => {
                "Name for the new file or folder.".into()
            }
            HoverTarget::ContentEmptyHint => {
                "Open a folder or a Markdown file using the toolbar buttons.".into()
            }
            HoverTarget::ContentPreviewPane => {
                "Rendered Markdown preview. Click a local .md link to open that file in the workspace."
                    .into()
            }
            HoverTarget::ContentEditorPane => {
                "Markdown source editor. Edits auto-save periodically and on exit.".into()
            }
            HoverTarget::ConflictReloadFromDisk => {
                "Discard local buffer and reload this file from disk.".into()
            }
            HoverTarget::ConflictKeepLocal => {
                "Keep your edited version; you can save it to overwrite the file on disk.".into()
            }
            HoverTarget::SidebarFile(p) => format!("Open {}.", p.display()),
            HoverTarget::SidebarFolderRow(p) => format!(
                "Expand or collapse {}, select it for new file or folder (header icons).",
                p.display()
            ),
            HoverTarget::SidebarHeaderNewFile => {
                "Create a new Markdown file in the selected folder.".into()
            }
            HoverTarget::SidebarHeaderNewFolder => {
                "Create a new subfolder in the selected folder.".into()
            }
            HoverTarget::SidebarProjectHeader => {
                "Open project or single-file workspace name.".into()
            }
            HoverTarget::SidebarEmptyState => {
                "Open a folder from the toolbar to browse files here.".into()
            }
        }
    }
}
