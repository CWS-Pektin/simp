use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use iced::keyboard;
use iced::widget::{column, container, operation, pane_grid, row, stack, Id};
use iced::window;
use iced::{Element, Fill, Size, Subscription, Task, Theme, Vector};

use crate::fs::{self, create_folder, create_markdown_file, load_file, save_atomic};
use crate::fs::watch::watch_subscription;
use crate::message::{CreateResult, Message, SettingsPreferenceChanged};
use crate::persist::{self, PersistState, ThemeMode, UserPreferences};
use crate::state::{
    DialogState, HoverTarget, OpenDocument, RootContext, Selection, SettingsCategory,
    SettingsNavTarget, StatusState, TreeNode,
};
use crate::ui::content::{EDITOR_SCROLLABLE_ID_STR, PREVIEW_SCROLLABLE_ID_STR};
use crate::ui::{
    view_content, view_dialog_overlay, view_menu, view_settings_content, view_settings_sidebar,
    view_sidebar, view_statusbar, view_toolbar,
};

pub use crate::state::ContentMode;

#[derive(Debug, Clone, Copy)]
pub enum PaneKind {
    Sidebar,
    Content,
}

pub struct App {
    pub root: Option<RootContext>,
    pub tree: Option<TreeNode>,
    pub expanded: HashSet<PathBuf>,
    pub selected: Option<Selection>,
    pub current_doc: Option<OpenDocument>,
    pub mode: ContentMode,
    /// Used when opening a file that has no per-file entry (also persisted as `content_mode`).
    default_content_mode: ContentMode,
    file_content_modes: HashMap<PathBuf, ContentMode>,
    /// Per-file preview scroll as relative Y (0..=1), keyed like `file_content_modes`.
    preview_scroll_rel_y: HashMap<PathBuf, f32>,
    editor_scroll_rel_y: HashMap<PathBuf, f32>,
    pub status: StatusState,
    pub pending_dialog: Option<DialogState>,
    pub theme_mode: ThemeMode,
    pub theme: Theme,
    pub watcher_id: u64,
    pub watch_path: Option<PathBuf>,
    pub panes: pane_grid::State<PaneKind>,
    last_edit: Option<Instant>,
    fs_rescan_pending: bool,
    last_fs_signal: Option<Instant>,
    /// Paths we recently wrote — ignore watcher noise for a short window.
    self_save_cooldown: HashMap<PathBuf, Instant>,
    pub conflict_banner: Option<String>,
    /// After an external change while the document is clean, reload from disk.
    pending_clean_reload: Option<PathBuf>,
    /// State to restore after the initial folder scan on startup.
    pending_restore: Option<PersistState>,

    pub show_settings: bool,
    pub settings_category: SettingsCategory,
    pub settings_draft_theme: ThemeMode,
    /// Applied preferences (preview/editor/general); persisted.
    pub preferences: UserPreferences,
    /// Edited in settings until Save.
    pub preferences_draft: UserPreferences,

    /// Last known windowed inner size (logical px); updated only when not fullscreen.
    window_width: f32,
    window_height: f32,
    window_fullscreen: bool,
    last_window_geometry_persist: Option<Instant>,

    /// Hovered control for the bottom status help strip (`None` when not over a tracked target).
    pub hover_target: Option<HoverTarget>,
}

impl Default for App {
    fn default() -> Self {
        // Sidebar starts at 15% of the window width
        let panes = pane_grid::State::with_configuration(pane_grid::Configuration::Split {
            axis: pane_grid::Axis::Vertical,
            ratio: 0.15,
            a: Box::new(pane_grid::Configuration::Pane(PaneKind::Sidebar)),
            b: Box::new(pane_grid::Configuration::Pane(PaneKind::Content)),
        });
        Self {
            root: None,
            tree: None,
            expanded: HashSet::new(),
            selected: None,
            current_doc: None,
            mode: ContentMode::Viewer,
            default_content_mode: ContentMode::Viewer,
            file_content_modes: HashMap::new(),
            preview_scroll_rel_y: HashMap::new(),
            editor_scroll_rel_y: HashMap::new(),
            status: StatusState::Idle,
            pending_dialog: None,
            theme_mode: ThemeMode::Light,
            theme: crate::ui::theme::build_theme(ThemeMode::Light),
            watcher_id: 0,
            watch_path: None,
            panes,
            last_edit: None,
            fs_rescan_pending: false,
            last_fs_signal: None,
            self_save_cooldown: HashMap::new(),
            conflict_banner: None,
            pending_clean_reload: None,
            pending_restore: None,
            show_settings: false,
            settings_category: SettingsCategory::General,
            settings_draft_theme: ThemeMode::Light,
            preferences: UserPreferences::default(),
            preferences_draft: UserPreferences::default(),
            window_width: 1024.0,
            window_height: 768.0,
            window_fullscreen: false,
            last_window_geometry_persist: None,
            hover_target: None,
        }
    }
}

impl App {
    pub fn new() -> (Self, Task<Message>) {
        let mut app = Self::default();
        let saved = persist::load();

        // Restore theme preference
        app.theme_mode = saved.theme_mode;
        app.theme = crate::ui::theme::build_theme(app.theme_mode);
        app.default_content_mode = saved.content_mode;
        app.file_content_modes = saved.file_content_modes.clone();
        app.preview_scroll_rel_y = saved.preview_scroll_rel_y.clone();
        app.editor_scroll_rel_y = saved.editor_scroll_rel_y.clone();
        app.mode = saved.content_mode;
        app.settings_draft_theme = app.theme_mode;
        app.preferences = saved.preferences.clone();
        app.preferences_draft = saved.preferences.clone();

        match (saved.window_width, saved.window_height) {
            (Some(w), Some(h))
                if w.is_finite()
                    && h.is_finite()
                    && w >= 320.0
                    && h >= 240.0 =>
            {
                app.window_width = w;
                app.window_height = h;
            }
            _ => {}
        }
        app.window_fullscreen = saved.window_fullscreen;

        // Only restore folder-mode state (not single-file sessions)
        if let Some(ref folder) = saved.root_folder {
            let p = folder.clone();
            let p2 = folder.clone();
            app.pending_restore = Some(saved);
            let task = Task::perform(
                async move {
                    tokio::task::spawn_blocking(move || fs::scan::build_tree(&p))
                        .await
                        .map_err(|e| e.to_string())
                        .and_then(|r| r.map_err(|e| e.to_string()))
                        .map(|tree| (tree, p2))
                },
                Message::FolderScanDone,
            );
            return (app, task);
        }

        (app, Task::none())
    }

    pub fn title(&self) -> String {
        let dirty = self
            .current_doc
            .as_ref()
            .map(|d| d.dirty || d.save_pending)
            .unwrap_or(false);
        format!(
            "MarkdownView{}",
            if dirty { " *" } else { "" }
        )
    }

    pub fn theme(&self) -> Theme {
        self.theme.clone()
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        if !matches!(&message, Message::HoverEnter(_) | Message::HoverLeave(_)) {
            if Self::message_clears_hover_tooltip(&message) {
                self.hover_target = None;
            }
        }

        match message {
            Message::HoverEnter(t) => {
                self.hover_target = Some(t);
                Task::none()
            }
            Message::HoverLeave(t) => {
                if self.hover_target.as_ref() == Some(&t) {
                    self.hover_target = None;
                }
                Task::none()
            }

            Message::SettingsPressed => {
                if self.pending_dialog.is_some() {
                    return Task::none();
                }
                if self.show_settings {
                    if self.settings_have_unsaved_changes() {
                        self.pending_dialog = Some(DialogState::UnsavedSettings {
                            target: SettingsNavTarget::ExitSettings,
                        });
                        Task::none()
                    } else {
                        self.show_settings = false;
                        self.settings_draft_theme = self.theme_mode;
                        self.preferences_draft = self.preferences.clone();
                        self.schedule_content_scroll_restore()
                    }
                } else {
                    self.show_settings = true;
                    self.settings_category = SettingsCategory::General;
                    self.settings_draft_theme = self.theme_mode;
                    self.preferences_draft = self.preferences.clone();
                    Task::none()
                }
            }
            Message::SettingsCategorySelected(cat) => {
                if !self.show_settings || self.pending_dialog.is_some() {
                    return Task::none();
                }
                if cat == self.settings_category {
                    return Task::none();
                }
                if self.settings_have_unsaved_changes() {
                    self.pending_dialog = Some(DialogState::UnsavedSettings {
                        target: SettingsNavTarget::Category(cat),
                    });
                    return Task::none();
                }
                self.settings_category = cat;
                self.settings_draft_theme = self.theme_mode;
                self.preferences_draft = self.preferences.clone();
                Task::none()
            }
            Message::SettingsDraftThemeSelected(mode) => {
                self.settings_draft_theme = mode;
                Task::none()
            }
            Message::SettingsPreferenceChanged(change) => {
                if !self.show_settings {
                    return Task::none();
                }
                match change {
                    SettingsPreferenceChanged::AutoSaveDebounceMs(ms) => {
                        self.preferences_draft.general.auto_save_debounce_ms = ms;
                    }
                    SettingsPreferenceChanged::ShowStatusHoverHints(v) => {
                        self.preferences_draft.general.show_status_hover_hints = v;
                    }
                    SettingsPreferenceChanged::PreviewMaxWidth(w) => {
                        self.preferences_draft.preview.max_content_width = w;
                    }
                    SettingsPreferenceChanged::PreviewBaseTextSize(s) => {
                        self.preferences_draft.preview.base_text_size = s;
                    }
                    SettingsPreferenceChanged::PreviewPaddingVertical(p) => {
                        self.preferences_draft.preview.padding_vertical = p;
                    }
                    SettingsPreferenceChanged::PreviewPaddingHorizontal(p) => {
                        self.preferences_draft.preview.padding_horizontal = p;
                    }
                    SettingsPreferenceChanged::EditorFontSize(s) => {
                        self.preferences_draft.editor.font_size = s;
                    }
                    SettingsPreferenceChanged::EditorPaddingHorizontal(p) => {
                        self.preferences_draft.editor.padding_horizontal = p;
                    }
                    SettingsPreferenceChanged::EditorPaddingVertical(p) => {
                        self.preferences_draft.editor.padding_vertical = p;
                    }
                }
                self.preferences_draft = self.preferences_draft.clone().sanitized();
                Task::none()
            }
            Message::SaveSettings => {
                if !self.show_settings || !self.settings_have_unsaved_changes() {
                    return Task::none();
                }
                self.apply_settings_save_from_draft();
                Task::none()
            }
            Message::UnsavedSettingsSave { target } => {
                self.apply_settings_save_from_draft();
                let restore_scroll = matches!(target, SettingsNavTarget::ExitSettings);
                self.complete_settings_navigation(target);
                self.pending_dialog = None;
                if restore_scroll {
                    self.schedule_content_scroll_restore()
                } else {
                    Task::none()
                }
            }
            Message::UnsavedSettingsDiscard { target } => {
                self.settings_draft_theme = self.theme_mode;
                self.preferences_draft = self.preferences.clone();
                let restore_scroll = matches!(target, SettingsNavTarget::ExitSettings);
                self.complete_settings_navigation(target);
                self.pending_dialog = None;
                if restore_scroll {
                    self.schedule_content_scroll_restore()
                } else {
                    Task::none()
                }
            }
            Message::UnsavedSettingsClose => {
                self.pending_dialog = None;
                Task::none()
            }

            Message::OpenFolderPressed => Task::perform(
                async {
                    rfd::AsyncFileDialog::new()
                        .pick_folder()
                        .await
                        .map(|h| h.path().to_path_buf())
                },
                Message::FolderChosen,
            ),

            Message::OpenFilePressed => Task::perform(
                async {
                    rfd::AsyncFileDialog::new()
                        .add_filter("Markdown", &["md", "MD", "Md"])
                        .pick_file()
                        .await
                        .map(|h| h.path().to_path_buf())
                },
                Message::FileChosen,
            ),

            Message::FolderChosen(None) => Task::none(),
            Message::FolderChosen(Some(path)) => {
                let p = path.clone();
                let p2 = path.clone();
                Task::perform(
                    async move {
                        tokio::task::spawn_blocking(move || fs::scan::build_tree(&p))
                            .await
                            .map_err(|e| e.to_string())
                            .and_then(|r| r.map_err(|e| e.to_string()))
                            .map(|tree| (tree, p2))
                    },
                    Message::FolderScanDone,
                )
            }

            Message::FolderScanDone(Err(e)) => {
                self.status = StatusState::Error(e);
                Task::none()
            }
            Message::FolderScanDone(Ok((tree, root_path))) => {
                self.root = Some(RootContext::Folder {
                    root_path: root_path.clone(),
                });
                self.tree = Some(tree);
                self.expanded.insert(root_path.clone());
                self.watcher_id = self.watcher_id.wrapping_add(1);
                self.watch_path = Some(root_path.clone());
                self.conflict_banner = None;
                self.pending_clean_reload = None;
                self.current_doc = None;
                self.status = StatusState::Idle;

                if let Some(restore) = self.pending_restore.take() {
                    // Restore persisted expanded set
                    for p in restore.expanded_set() {
                        self.expanded.insert(p);
                    }

                    // Prefer restoring the last-open file (stay on same doc after restart)
                    if let Some(ref fp) = restore.open_file {
                        if fp.is_file()
                            && fs::scan::is_markdown_file(fp)
                            && fs::path_within_root(fp, &root_path)
                        {
                            self.selected = Some(Selection::File(fp.clone()));
                            let path = fp.clone();
                            let path2 = fp.clone();
                            return Task::perform(
                                async move {
                                    load_file(&path).await.map_err(|e| e.to_string())
                                },
                                move |r| Message::FileOpenResult(path2, r),
                            );
                        }
                    }

                    if let Some(cat) = restore.selected_category {
                        self.selected = Some(Selection::Category(cat));
                    } else {
                        self.selected = Some(Selection::Category(root_path));
                    }
                } else {
                    // Fresh user-opened folder: select root category, save state
                    self.selected = Some(Selection::Category(root_path));
                    self.save_persist();
                }

                Task::none()
            }

            Message::InitialFileLoaded(path, res) => {
                let mode = self.mode_for_opened_file(&path);
                self.apply_loaded_file(path, res, mode);
                self.schedule_content_scroll_restore()
            }

            Message::FileChosen(None) => Task::none(),
            Message::FileChosen(Some(path)) => {
                if !fs::scan::is_markdown_file(&path) {
                    self.status = StatusState::Error("Not a Markdown file".into());
                    return Task::none();
                }
                let parent = path
                    .parent()
                    .map(Path::to_path_buf)
                    .unwrap_or_else(|| PathBuf::from("."));
                let name = path
                    .file_name()
                    .map(|s| s.to_string_lossy().into_owned())
                    .unwrap_or_default();

                let file_node = TreeNode {
                    path: path.clone(),
                    name,
                    kind: crate::state::NodeKind::MarkdownFile,
                    children: vec![],
                };
                let root_node = TreeNode {
                    path: parent.clone(),
                    name: parent
                        .file_name()
                        .map(|s| s.to_string_lossy().into_owned())
                        .unwrap_or_else(|| parent.display().to_string()),
                    kind: crate::state::NodeKind::Category,
                    children: vec![file_node],
                };

                self.root = Some(RootContext::SingleFile {
                    file_path: path.clone(),
                });
                self.tree = Some(root_node);
                self.expanded.insert(parent);
                self.watcher_id = self.watcher_id.wrapping_add(1);
                self.watch_path = path.parent().map(Path::to_path_buf);
                self.selected = Some(Selection::File(path.clone()));
                self.conflict_banner = None;

                let path2 = path.clone();
                Task::perform(
                    async move { load_file(&path).await.map_err(|e| e.to_string()) },
                    move |r| Message::InitialFileLoaded(path2, r),
                )
            }

            Message::FolderRowPressed(p) => {
                self.selected = Some(Selection::Category(p.clone()));
                if self.expanded.contains(&p) {
                    self.expanded.remove(&p);
                } else {
                    self.expanded.insert(p);
                }
                self.save_persist();
                Task::none()
            }

            Message::FileSelected(p) => {
                if let Some(doc) = &self.current_doc {
                    if doc.path == p && !doc.dirty {
                        self.selected = Some(Selection::File(p.clone()));
                        return Task::none();
                    }
                }
                self.selected = Some(Selection::File(p.clone()));
                self.conflict_banner = None;
                let p2 = p.clone();
                Task::perform(
                    async move { load_file(&p).await.map_err(|e| e.to_string()) },
                    move |r| Message::FileOpenResult(p2, r),
                )
            }

            Message::FileOpenResult(path, Ok((text, mtime))) => {
                self.mode = self.mode_for_opened_file(&path);
                self.current_doc = Some(OpenDocument::new(path, text, mtime));
                self.status = StatusState::Idle;
                self.save_persist();
                self.schedule_content_scroll_restore()
            }
            Message::FileOpenResult(path, Err(e)) => {
                self.status = StatusState::Error(e);
                self.current_doc = None;
                // Restore startup / sidebar selection if the file vanished or failed to load
                if let Some(RootContext::Folder { root_path }) = &self.root {
                    if matches!(self.selected, Some(Selection::File(ref p)) if p == &path) {
                        let cat = path
                            .parent()
                            .filter(|p| p.starts_with(root_path.as_path()))
                            .map(Path::to_path_buf)
                            .unwrap_or_else(|| root_path.clone());
                        self.selected = Some(Selection::Category(cat));
                    }
                }
                self.save_persist();
                Task::none()
            }

            Message::ModeViewer => {
                if let Some(path) = self.current_doc.as_ref().map(|d| d.path.clone()) {
                    if self.mode == ContentMode::Viewer {
                        return Task::none();
                    }
                    self.mode = ContentMode::Viewer;
                    self.remember_file_content_mode(&path, self.mode);
                    self.sync_scroll_for_mode_switch(&path);
                    self.save_persist();
                    return self.schedule_preview_scroll_restore();
                }
                Task::none()
            }
            Message::ModeEditor => {
                if let Some(path) = self.current_doc.as_ref().map(|d| d.path.clone()) {
                    if self.mode == ContentMode::Editor {
                        return Task::none();
                    }
                    self.mode = ContentMode::Editor;
                    self.remember_file_content_mode(&path, self.mode);
                    self.sync_scroll_for_mode_switch(&path);
                    self.save_persist();
                    return self.schedule_editor_scroll_restore();
                }
                Task::none()
            }
            Message::ToggleMode => {
                if let Some(path) = self.current_doc.as_ref().map(|d| d.path.clone()) {
                    self.mode = if self.mode == ContentMode::Viewer {
                        ContentMode::Editor
                    } else {
                        ContentMode::Viewer
                    };
                    self.remember_file_content_mode(&path, self.mode);
                    self.sync_scroll_for_mode_switch(&path);
                    self.save_persist();
                    if self.mode == ContentMode::Viewer {
                        return self.schedule_preview_scroll_restore();
                    }
                    return self.schedule_editor_scroll_restore();
                }
                Task::none()
            }

            Message::PreviewScrolledRel(rel_y) => {
                if let Some(rel_y) = rel_y {
                    if rel_y.is_finite() && self.mode == ContentMode::Viewer {
                        if let Some(doc) = &self.current_doc {
                            let key = fs::content_mode_key(&doc.path);
                            self.preview_scroll_rel_y
                                .insert(key, rel_y.clamp(0.0, 1.0));
                        }
                    }
                }
                Task::none()
            }

            Message::EditorOuterScroll(delta) => {
                let v = editor_wheel_vector(delta);
                operation::scroll_by::<Message>(
                    Id::new(EDITOR_SCROLLABLE_ID_STR),
                    operation::AbsoluteOffset { x: v.x, y: v.y },
                )
            }

            Message::EditorScrolledRel(rel_y) => {
                if let Some(rel_y) = rel_y {
                    if rel_y.is_finite() && self.mode == ContentMode::Editor {
                        if let Some(doc) = &self.current_doc {
                            let key = fs::content_mode_key(&doc.path);
                            self.editor_scroll_rel_y
                                .insert(key, rel_y.clamp(0.0, 1.0));
                        }
                    }
                }
                Task::none()
            }

            Message::EditorAction(action) => {
                if matches!(
                    &action,
                    iced::widget::text_editor::Action::Scroll { .. }
                ) {
                    return Task::none();
                }
                let Some(doc) = self.current_doc.as_mut() else {
                    return Task::none();
                };
                let is_edit = action.is_edit();
                doc.editor.perform(action);
                if is_edit {
                    doc.sync_text_from_editor();
                    doc.dirty = true;
                    doc.save_pending = true;
                    self.last_edit = Some(Instant::now());
                    self.pending_clean_reload = None;
                    self.status = StatusState::Idle;
                }
                Task::none()
            }

            Message::SaveTick => {
                let Some(doc) = self.current_doc.as_mut() else {
                    return Task::none();
                };
                if !doc.dirty && !doc.save_pending {
                    return Task::none();
                }
                let last = self.last_edit.unwrap_or_else(Instant::now);
                let debounce = self
                    .preferences
                    .general
                    .auto_save_debounce_ms
                    .max(250);
                if last.elapsed() < Duration::from_millis(debounce as u64) {
                    return Task::none();
                }
                let path = doc.path.clone();
                let text = doc.text.clone();
                doc.save_pending = false;
                self.status = StatusState::Saving;
                Task::perform(
                    async move { save_atomic(&path, &text).await.map_err(|e| e.to_string()) },
                    Message::SaveCompleted,
                )
            }

            Message::SaveCompleted(Ok(())) => {
                if let Some(doc) = self.current_doc.as_mut() {
                    doc.dirty = false;
                    doc.last_saved_text = Some(doc.text.clone());
                    if let Ok(m) = std::fs::metadata(&doc.path).and_then(|m| m.modified()) {
                        doc.last_disk_mtime = Some(m);
                    }
                    self.self_save_cooldown
                        .insert(doc.path.clone(), Instant::now());
                }
                self.status = StatusState::Saved;
                Task::none()
            }
            Message::SaveCompleted(Err(e)) => {
                if let Some(doc) = self.current_doc.as_mut() {
                    doc.save_pending = true;
                }
                self.status = StatusState::Error(e);
                Task::none()
            }

            Message::FsChange(path) => {
                if self.is_self_save_noise(&path) {
                    return Task::none();
                }
                self.fs_rescan_pending = true;
                self.last_fs_signal = Some(Instant::now());
                Task::none()
            }

            Message::WatcherInitFailed(e) => {
                self.status = StatusState::Error(format!("Watcher: {e}"));
                Task::none()
            }

            Message::TreeRescanTick => {
                if self.fs_rescan_pending {
                    if let Some(last) = self.last_fs_signal {
                        if last.elapsed() >= Duration::from_millis(200) {
                            self.fs_rescan_pending = false;
                            self.apply_fs_rescan();
                        }
                    } else {
                        self.fs_rescan_pending = false;
                    }
                }

                if let Some(p) = &self.pending_clean_reload {
                    let should = self
                        .current_doc
                        .as_ref()
                        .map(|d| d.path == *p && !d.dirty)
                        .unwrap_or(false);
                    if should {
                        let p = self.pending_clean_reload.take().unwrap();
                        let p2 = p.clone();
                        return Task::perform(
                            async move { load_file(&p).await.map_err(|e| e.to_string()) },
                            move |r| Message::FileOpenResult(p2, r),
                        );
                    }
                }

                Task::none()
            }

            Message::DocumentReloadCompleted(path, Ok((text, mtime))) => {
                let mut ok = false;
                if let Some(doc) = self.current_doc.as_mut() {
                    if doc.path == path {
                        doc.replace_content(text, mtime);
                        self.conflict_banner = None;
                        self.status = StatusState::Idle;
                        ok = true;
                    }
                }
                if ok {
                    self.schedule_content_scroll_restore()
                } else {
                    Task::none()
                }
            }
            Message::DocumentReloadCompleted(_, Err(e)) => {
                self.status = StatusState::Error(e);
                Task::none()
            }

            Message::NewMarkdownFilePressed => {
                let Some(parent) = self.creation_target_dir() else {
                    self.status = StatusState::Error("Select a folder first".into());
                    return Task::none();
                };
                self.pending_dialog = Some(DialogState::NewMarkdownFile {
                    parent,
                    input: String::new(),
                });
                Task::none()
            }

            Message::NewFolderPressed => {
                let Some(parent) = self.creation_target_dir() else {
                    self.status = StatusState::Error("Select a folder first".into());
                    return Task::none();
                };
                self.pending_dialog = Some(DialogState::NewFolder {
                    parent,
                    input: String::new(),
                });
                Task::none()
            }

            Message::CreateDialogInputChanged(s) => {
                if let Some(d) = self.pending_dialog.as_mut() {
                    match d {
                        DialogState::NewMarkdownFile { input, .. }
                        | DialogState::NewFolder { input, .. } => {
                            *input = s;
                        }
                        DialogState::UnsavedSettings { .. } => {}
                    }
                }
                Task::none()
            }

            Message::CancelCreateDialog => {
                self.pending_dialog = None;
                Task::none()
            }

            Message::ConfirmCreateDialog => {
                let Some(dialog) = self.pending_dialog.take() else {
                    return Task::none();
                };
                match dialog {
                    DialogState::NewMarkdownFile { parent, input } => {
                        let name = input.trim();
                        if let Err(e) = validate_name(name) {
                            self.status = StatusState::Error(e);
                            return Task::none();
                        }
                        let mut path = parent.join(name);
                        if !path
                            .extension()
                            .map(|e| e.to_string_lossy().eq_ignore_ascii_case("md"))
                            .unwrap_or(false)
                        {
                            path.set_extension("md");
                        }
                        if path.exists() {
                            self.status = StatusState::Error("A file with that name already exists".into());
                            return Task::none();
                        }
                        Task::perform(
                            async move {
                                create_markdown_file(&path)
                                    .await
                                    .map_err(|e| e.to_string())?;
                                Ok(CreateResult {
                                    path,
                                    is_file: true,
                                })
                            },
                            Message::CreateCompleted,
                        )
                    }
                    DialogState::NewFolder { parent, input } => {
                        let name = input.trim();
                        if let Err(e) = validate_name(name) {
                            self.status = StatusState::Error(e);
                            return Task::none();
                        }
                        let path = parent.join(name);
                        if path.exists() {
                            self.status =
                                StatusState::Error("A folder with that name already exists".into());
                            return Task::none();
                        }
                        Task::perform(
                            async move {
                                create_folder(&path)
                                    .await
                                    .map_err(|e| e.to_string())?;
                                Ok(CreateResult {
                                    path,
                                    is_file: false,
                                })
                            },
                            Message::CreateCompleted,
                        )
                    }
                    DialogState::UnsavedSettings { .. } => {
                        self.pending_dialog = Some(dialog);
                        Task::none()
                    }
                }
            }

            Message::CreateCompleted(Ok(result)) => {
                self.rescan_tree_after_change();
                if result.is_file {
                    let parent = result
                        .path
                        .parent()
                        .map(Path::to_path_buf)
                        .unwrap_or_else(|| PathBuf::from("."));
                    self.expanded.insert(parent);
                    self.selected = Some(Selection::File(result.path.clone()));
                    self.mode = ContentMode::Editor;
                    self.remember_file_content_mode(&result.path, ContentMode::Editor);
                    self.current_doc = Some(OpenDocument::new(result.path, String::new(), None));
                    self.status = StatusState::Idle;
                } else {
                    self.expanded.insert(
                        result
                            .path
                            .parent()
                            .unwrap_or_else(|| result.path.as_path())
                            .to_path_buf(),
                    );
                    self.expanded.insert(result.path.clone());
                    self.selected = Some(Selection::Category(result.path));
                    self.status = StatusState::Idle;
                }
                Task::none()
            }
            Message::CreateCompleted(Err(e)) => {
                self.status = StatusState::Error(e);
                Task::none()
            }

            Message::ConflictReloadFromDisk => {
                let Some(doc) = self.current_doc.as_ref() else {
                    self.conflict_banner = None;
                    return Task::none();
                };
                let path = doc.path.clone();
                let path2 = path.clone();
                Task::perform(
                    async move { load_file(&path).await.map_err(|e| e.to_string()) },
                    move |r| Message::DocumentReloadCompleted(path2, r),
                )
            }

            Message::ConflictKeepLocal => {
                self.conflict_banner = None;
                if let Some(doc) = self.current_doc.as_mut() {
                    doc.dirty = true;
                    doc.save_pending = true;
                    self.last_edit = Some(Instant::now());
                }
                self.status = StatusState::Idle;
                Task::none()
            }

            Message::LinkClicked(uri) => {
                let Some(doc) = self.current_doc.as_ref() else {
                    return Task::none();
                };
                let Some(target) = fs::resolve_local_markdown_link(&uri, &doc.path) else {
                    return Task::none();
                };
                if !target.exists() {
                    self.status = StatusState::Error(format!(
                        "Linked path not found: {}",
                        target.display()
                    ));
                    return Task::none();
                }
                if !fs::scan::is_markdown_file(&target) {
                    self.status = StatusState::Error(format!(
                        "Link is not a Markdown file: {}",
                        target.display()
                    ));
                    return Task::none();
                }
                let allowed = match &self.root {
                    Some(RootContext::Folder { root_path }) => {
                        fs::path_within_root(&target, root_path)
                    }
                    Some(RootContext::SingleFile { file_path }) => file_path
                        .parent()
                        .map(|parent| fs::path_within_root(&target, parent))
                        .unwrap_or(false),
                    None => doc
                        .path
                        .parent()
                        .map(|parent| fs::path_within_root(&target, parent))
                        .unwrap_or(false),
                };
                if !allowed {
                    self.status = StatusState::Error(
                        "That link points outside the allowed folder for this workspace.".into(),
                    );
                    return Task::none();
                }
                if let Some(d) = &self.current_doc {
                    if d.path == target && !d.dirty {
                        self.selected = Some(Selection::File(target.clone()));
                        self.expand_sidebar_ancestors_of(&target);
                        return Task::none();
                    }
                }
                self.expand_sidebar_ancestors_of(&target);
                self.selected = Some(Selection::File(target.clone()));
                self.conflict_banner = None;
                let p = target.clone();
                Task::perform(
                    async move { load_file(&p).await.map_err(|e| e.to_string()) },
                    move |r| Message::FileOpenResult(target, r),
                )
            }

            Message::PaneResized(pane_grid::ResizeEvent { split, ratio }) => {
                // Clamp sidebar: minimum ~18%, maximum ~35%
                self.panes.resize(split, ratio.clamp(0.18, 0.35));
                Task::none()
            }

            Message::WindowResized(id, size) => {
                window::mode(id).map(move |m| Message::WindowGeometryPersist {
                    size,
                    fullscreen: m == window::Mode::Fullscreen,
                })
            }

            Message::WindowGeometryPersist { size, fullscreen } => {
                self.window_fullscreen = fullscreen;
                if !fullscreen {
                    if size.width.is_finite()
                        && size.height.is_finite()
                        && size.width >= 320.0
                        && size.height >= 240.0
                    {
                        self.window_width = size.width;
                        self.window_height = size.height;
                    }
                }
                let now = Instant::now();
                let should_persist = self
                    .last_window_geometry_persist
                    .map(|t| now.duration_since(t) >= Duration::from_millis(450))
                    .unwrap_or(true);
                if should_persist {
                    self.last_window_geometry_persist = Some(now);
                    self.save_persist();
                }
                Task::none()
            }

            Message::WindowCloseRequested(id) => {
                if let Some(doc) = self.current_doc.as_mut() {
                    doc.sync_text_from_editor();
                    if doc.dirty {
                        match std::fs::write(&doc.path, doc.text.as_bytes()) {
                            Ok(()) => {
                                doc.dirty = false;
                                doc.save_pending = false;
                                doc.last_saved_text = Some(doc.text.clone());
                                if let Ok(m) = std::fs::metadata(&doc.path).and_then(|m| m.modified()) {
                                    doc.last_disk_mtime = Some(m);
                                }
                                self.self_save_cooldown.insert(doc.path.clone(), Instant::now());
                                self.status = StatusState::Saved;
                            }
                            Err(e) => {
                                self.status = StatusState::Error(e.to_string());
                            }
                        }
                    }
                }
                window::mode(id).then(move |m| {
                    if m == window::Mode::Fullscreen {
                        Task::done(Message::CloseWindowGeometry {
                            fullscreen: true,
                            size: None,
                        })
                    } else {
                        window::size(id).map(move |s| Message::CloseWindowGeometry {
                            fullscreen: false,
                            size: Some(s),
                        })
                    }
                })
            }

            Message::CloseWindowGeometry { fullscreen, size } => {
                self.window_fullscreen = fullscreen;
                if let Some(s) = size {
                    if s.width.is_finite()
                        && s.height.is_finite()
                        && s.width >= 320.0
                        && s.height >= 240.0
                    {
                        self.window_width = s.width;
                        self.window_height = s.height;
                    }
                }
                self.save_persist();
                Task::none()
            }
        }
    }

    fn apply_loaded_file(
        &mut self,
        path: PathBuf,
        res: Result<(String, Option<std::time::SystemTime>), String>,
        mode: ContentMode,
    ) {
        match res {
            Ok((text, mtime)) => {
                self.current_doc = Some(OpenDocument::new(path, text, mtime));
                self.mode = mode;
                self.status = StatusState::Idle;
            }
            Err(e) => {
                self.status = StatusState::Error(e);
                self.current_doc = None;
            }
        }
    }

    fn mode_for_opened_file(&self, path: &Path) -> ContentMode {
        let key = fs::content_mode_key(path);
        self.file_content_modes
            .get(&key)
            .copied()
            .unwrap_or(self.default_content_mode)
    }

    fn remember_file_content_mode(&mut self, path: &Path, mode: ContentMode) {
        let key = fs::content_mode_key(path);
        self.file_content_modes.insert(key, mode);
        self.default_content_mode = mode;
    }

    /// When switching preview ↔ editor on the **same** file, align both scroll maps so the
    /// incoming mode shows the same region as the outgoing mode. Not used on file open.
    fn sync_scroll_for_mode_switch(&mut self, path: &Path) {
        let key = fs::content_mode_key(path);
        match self.mode {
            ContentMode::Viewer => {
                let y = self
                    .editor_scroll_rel_y
                    .get(&key)
                    .copied()
                    .filter(|v| v.is_finite())
                    .unwrap_or(0.0);
                self.preview_scroll_rel_y
                    .insert(key, y.clamp(0.0, 1.0));
            }
            ContentMode::Editor => {
                let y = self
                    .preview_scroll_rel_y
                    .get(&key)
                    .copied()
                    .filter(|v| v.is_finite())
                    .unwrap_or(0.0);
                self.editor_scroll_rel_y
                    .insert(key, y.clamp(0.0, 1.0));
            }
        }
    }

    fn schedule_content_scroll_restore(&mut self) -> Task<Message> {
        match self.mode {
            ContentMode::Viewer => self.schedule_preview_scroll_restore(),
            ContentMode::Editor => self.schedule_editor_scroll_restore(),
        }
    }

    /// Snap preview scroll from [`Self::preview_scroll_rel_y`] (widget operation runs after layout).
    fn schedule_preview_scroll_restore(&mut self) -> Task<Message> {
        let Some(doc) = self.current_doc.as_ref() else {
            return Task::none();
        };
        if self.mode != ContentMode::Viewer {
            return Task::none();
        }
        let key = fs::content_mode_key(&doc.path);
        let Some(rel_y) = self.preview_scroll_rel_y.get(&key).copied() else {
            return Task::none();
        };
        if !rel_y.is_finite() {
            return Task::none();
        }
        let y = rel_y.clamp(0.0, 1.0);
        operation::snap_to::<Message>(
            Id::new(PREVIEW_SCROLLABLE_ID_STR),
            operation::RelativeOffset {
                x: None,
                y: Some(y),
            },
        )
    }

    fn schedule_editor_scroll_restore(&mut self) -> Task<Message> {
        let Some(doc) = self.current_doc.as_ref() else {
            return Task::none();
        };
        if self.mode != ContentMode::Editor {
            return Task::none();
        }
        let key = fs::content_mode_key(&doc.path);
        let Some(rel_y) = self.editor_scroll_rel_y.get(&key).copied() else {
            return Task::none();
        };
        if !rel_y.is_finite() {
            return Task::none();
        }
        let y = rel_y.clamp(0.0, 1.0);
        operation::snap_to::<Message>(
            Id::new(EDITOR_SCROLLABLE_ID_STR),
            operation::RelativeOffset {
                x: None,
                y: Some(y),
            },
        )
    }

    fn creation_target_dir(&self) -> Option<PathBuf> {
        match &self.selected {
            Some(Selection::Category(p)) => Some(p.clone()),
            Some(Selection::File(p)) => p.parent().map(Path::to_path_buf),
            None => match &self.root {
                Some(RootContext::Folder { root_path }) => Some(root_path.clone()),
                Some(RootContext::SingleFile { file_path }) => {
                    file_path.parent().map(Path::to_path_buf)
                }
                None => None,
            },
        }
    }

    fn is_self_save_noise(&self, path: &Path) -> bool {
        if let Some(t) = self.self_save_cooldown.get(path) {
            if t.elapsed() < Duration::from_millis(800) {
                return true;
            }
        }
        false
    }

    fn prune_self_save_cooldown(&mut self) {
        self.self_save_cooldown
            .retain(|_, t| t.elapsed() < Duration::from_millis(800));
    }

    fn rescan_tree_after_change(&mut self) {
        let Some(RootContext::Folder { root_path }) = &self.root else {
            return;
        };
        let root = root_path.clone();
        if let Ok(tree) = fs::scan::build_tree(&root) {
            self.tree = Some(tree);
        }
    }

    fn apply_fs_rescan(&mut self) {
        self.prune_self_save_cooldown();

        let current_path = self.current_doc.as_ref().map(|d| d.path.clone());

        match &self.root {
            Some(RootContext::Folder { root_path }) => {
                let root = root_path.clone();
                match fs::scan::build_tree(&root) {
                    Ok(tree) => {
                        self.tree = Some(tree);
                        self.expanded.retain(|p| p.exists());
                    }
                    Err(e) => {
                        self.status = StatusState::Error(e.to_string());
                    }
                }
            }
            Some(RootContext::SingleFile { file_path }) => {
                if !file_path.exists() {
                    self.current_doc = None;
                    self.selected = None;
                    self.status = StatusState::Error("The open file was deleted.".into());
                    self.watch_path = None;
                }
            }
            None => {}
        }

        if let Some(ref p) = current_path {
            if !p.exists() {
                self.current_doc = None;
                self.selected = self
                    .tree
                    .as_ref()
                    .and_then(fs::scan::default_open_path)
                    .map(Selection::File)
                    .or_else(|| {
                        self.root.as_ref().map(|r| match r {
                            RootContext::Folder { root_path } => {
                                Selection::Category(root_path.clone())
                            }
                            RootContext::SingleFile { file_path } => {
                                Selection::File(file_path.clone())
                            }
                        })
                    });
                self.status = StatusState::Error("Current file was removed.".into());
                return;
            }

            if let Some(doc) = self.current_doc.as_ref() {
                if doc.path == *p {
                    let disk_mtime = std::fs::metadata(p).and_then(|m| m.modified()).ok();
                    let changed_on_disk = match (disk_mtime, doc.last_disk_mtime) {
                        (Some(a), Some(b)) => a != b,
                        (Some(_), None) => true,
                        _ => false,
                    };
                    if changed_on_disk && !self.is_self_save_noise(p) {
                        if doc.dirty {
                            self.conflict_banner = Some(
                                "External changes detected while you have unsaved edits.".into(),
                            );
                            self.status = StatusState::Conflict(
                                "Resolve conflict to continue syncing".into(),
                            );
                        } else {
                            self.pending_clean_reload = Some(p.clone());
                        }
                    }
                }
            }
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        let save_tick = if self
            .current_doc
            .as_ref()
            .map(|d| d.dirty || d.save_pending)
            .unwrap_or(false)
        {
            iced::time::every(Duration::from_millis(50)).map(|_| Message::SaveTick)
        } else {
            Subscription::none()
        };

        let fs_tick = iced::time::every(Duration::from_millis(100)).map(|_| Message::TreeRescanTick);

        let watch = if let Some(ref wp) = self.watch_path {
            watch_subscription(self.watcher_id, wp.clone())
        } else {
            Subscription::none()
        };

        // Non-capturing closure required by Subscription::filter_map
        let keys = keyboard::listen().filter_map(|event| match event {
            keyboard::Event::KeyPressed {
                key: k,
                modifiers,
                ..
            } => match k.as_ref() {
                keyboard::Key::Character(s) if modifiers.control() => {
                    let ch = s.chars().next().unwrap_or('\0').to_ascii_lowercase();
                    if ch == 'o' {
                        if modifiers.shift() {
                            Some(Message::OpenFolderPressed)
                        } else {
                            Some(Message::OpenFilePressed)
                        }
                    } else if ch == 'e' {
                        Some(Message::ToggleMode)
                    } else {
                        None
                    }
                }
                keyboard::Key::Named(keyboard::key::Named::Escape) => {
                    Some(Message::CancelCreateDialog)
                }
                _ => None,
            },
            _ => None,
        });

        let close_flush =
            window::close_requests().map(Message::WindowCloseRequested);

        let resize = window::resize_events().map(|(id, size)| Message::WindowResized(id, size));

        Subscription::batch([save_tick, fs_tick, watch, keys, close_flush, resize])
    }

    pub fn view(&self) -> Element<'_, Message> {
        use crate::ui::theme as t;
        let mode = self.theme_mode;

        let grid = pane_grid(&self.panes, move |_pane, kind, _maximized| {
            let content: Element<'_, Message> = match kind {
                PaneKind::Sidebar => {
                    let inner: Element<'_, Message> = if self.show_settings {
                        view_settings_sidebar(self).into()
                    } else {
                        view_sidebar(
                            self,
                            self.tree.as_ref(),
                            &self.expanded,
                            self.selected.as_ref(),
                        )
                        .into()
                    };
                    container(inner)
                        .style(move |theme| t::sidebar_pane(theme, mode))
                        .width(Fill)
                        .height(Fill)
                        .into()
                }
                PaneKind::Content => {
                    let inner: Element<'_, Message> = if self.show_settings {
                        view_settings_content(self).into()
                    } else {
                        view_content(self).into()
                    };
                    container(inner)
                        .style(move |theme| t::content_pane(theme, mode))
                        .width(Fill)
                        .height(Fill)
                        .into()
                }
            };
            pane_grid::Content::new(content)
        })
        .on_resize(6, Message::PaneResized)
        .width(Fill)
        .height(Fill);

        let toolbar = container(view_toolbar(self))
            .style(move |theme| t::toolbar_bar(theme, mode))
            .width(Fill);

        let menu_bar = view_menu(self);

        let statusbar = container(view_statusbar(self))
            .style(move |theme| t::status_bar(theme, mode))
            .width(Fill);

        let body: Element<'_, Message> = match &self.pending_dialog {
            Some(d)
                if matches!(
                    d,
                    DialogState::NewMarkdownFile { .. } | DialogState::NewFolder { .. }
                ) =>
            {
                stack![
                    column![
                        toolbar,
                        menu_bar,
                        grid,
                        statusbar,
                    ],
                    crate::ui::dialogs::view_create_dialog_modal(self, d),
                ]
                .width(Fill)
                .height(Fill)
                .into()
            }
            Some(d) => column![
                toolbar,
                menu_bar,
                row![
                    grid,
                    container(view_dialog_overlay(self, d))
                        .max_width(340)
                        .height(Fill)
                        .style(move |theme| t::dialog_card(theme, mode)),
                ]
                .height(Fill),
                statusbar,
            ]
            .into(),
            None => column![
                toolbar,
                menu_bar,
                grid,
                statusbar,
            ]
            .into(),
        };

        body
    }

    pub fn settings_have_unsaved_changes(&self) -> bool {
        self.settings_draft_theme != self.theme_mode
            || self.preferences_draft != self.preferences
    }

    fn apply_settings_save_from_draft(&mut self) {
        self.theme_mode = self.settings_draft_theme;
        self.theme = crate::ui::theme::build_theme(self.theme_mode);
        self.preferences = self.preferences_draft.clone().sanitized();
        self.preferences_draft = self.preferences.clone();
        self.save_persist();
    }

    fn complete_settings_navigation(&mut self, target: SettingsNavTarget) {
        match target {
            SettingsNavTarget::Category(c) => {
                self.settings_category = c;
                self.settings_draft_theme = self.theme_mode;
                self.preferences_draft = self.preferences.clone();
            }
            SettingsNavTarget::ExitSettings => {
                self.show_settings = false;
                self.settings_draft_theme = self.theme_mode;
                self.preferences_draft = self.preferences.clone();
            }
        }
    }

    /// Persist the current folder-mode state to disk (no-op in single-file mode).
    fn save_persist(&self) {
        let (root_folder, selected_category, open_file) = match &self.root {
            Some(RootContext::Folder { ref root_path }) => {
                let sel = match &self.selected {
                    Some(Selection::Category(p)) => Some(p.clone()),
                    _ => None,
                };
                let open = self
                    .current_doc
                    .as_ref()
                    .map(|d| d.path.clone())
                    .filter(|p| fs::path_within_root(p, root_path) && fs::scan::is_markdown_file(p));
                (Some(root_path.clone()), sel, open)
            }
            _ => (None, None, None),
        };

        let file_content_modes = match &self.root {
            Some(RootContext::Folder { root_path }) => self
                .file_content_modes
                .iter()
                .filter(|(p, _)| fs::path_within_root(p, root_path))
                .map(|(k, v)| (k.clone(), *v))
                .collect(),
            _ => HashMap::new(),
        };

        let preview_scroll_rel_y = match &self.root {
            Some(RootContext::Folder { root_path }) => self
                .preview_scroll_rel_y
                .iter()
                .filter(|(p, _)| fs::path_within_root(p, root_path))
                .filter(|(_, y)| y.is_finite())
                .map(|(k, v)| (k.clone(), (*v).clamp(0.0, 1.0)))
                .collect(),
            _ => HashMap::new(),
        };

        let editor_scroll_rel_y = match &self.root {
            Some(RootContext::Folder { root_path }) => self
                .editor_scroll_rel_y
                .iter()
                .filter(|(p, _)| fs::path_within_root(p, root_path))
                .filter(|(_, y)| y.is_finite())
                .map(|(k, v)| (k.clone(), (*v).clamp(0.0, 1.0)))
                .collect(),
            _ => HashMap::new(),
        };

        persist::save(&PersistState {
            root_folder,
            expanded: self.expanded.iter().cloned().collect(),
            selected_category,
            theme_mode: self.theme_mode,
            content_mode: self.default_content_mode,
            file_content_modes,
            preview_scroll_rel_y,
            editor_scroll_rel_y,
            open_file,
            window_width: Some(self.window_width),
            window_height: Some(self.window_height),
            window_fullscreen: self.window_fullscreen,
            preferences: self.preferences.clone(),
        });
    }

    /// Expand folder nodes so `file_path` is reachable in the project tree.
    fn expand_sidebar_ancestors_of(&mut self, file_path: &Path) {
        let Some(RootContext::Folder { root_path }) = &self.root else {
            return;
        };
        let Some(mut p) = file_path.parent().map(Path::to_path_buf) else {
            return;
        };
        while p != *root_path && fs::path_within_root(&p, root_path) {
            self.expanded.insert(p.clone());
            let Some(par) = p.parent() else {
                break;
            };
            if par == p {
                break;
            }
            p = par.to_path_buf();
        }
    }

    fn message_clears_hover_tooltip(message: &Message) -> bool {
        matches!(
            message,
            Message::SettingsPressed
                | Message::SettingsCategorySelected(_)
                | Message::OpenFolderPressed
                | Message::OpenFilePressed
                | Message::FolderChosen(_)
                | Message::FileChosen(_)
                | Message::FolderScanDone(_)
                | Message::InitialFileLoaded(_, _)
                | Message::FileOpenResult(_, _)
                | Message::FolderRowPressed(_)
                | Message::FileSelected(_)
                | Message::LinkClicked(_)
                | Message::ModeViewer
                | Message::ModeEditor
                | Message::ToggleMode
                | Message::NewMarkdownFilePressed
                | Message::NewFolderPressed
                | Message::ConfirmCreateDialog
                | Message::CancelCreateDialog
                | Message::CreateCompleted(_)
                | Message::DocumentReloadCompleted(_, _)
                | Message::ConflictReloadFromDisk
                | Message::ConflictKeepLocal
                | Message::UnsavedSettingsSave { .. }
                | Message::UnsavedSettingsDiscard { .. }
                | Message::UnsavedSettingsClose
                | Message::SaveSettings
                | Message::SettingsPreferenceChanged(_)
        )
    }
}

fn editor_wheel_vector(delta: iced::mouse::ScrollDelta) -> Vector {
    match delta {
        iced::mouse::ScrollDelta::Lines { x, y, .. } => Vector::new(x, y) * -60.0,
        iced::mouse::ScrollDelta::Pixels { x, y } => Vector::new(-x, -y),
    }
}

fn validate_name(name: &str) -> Result<(), String> {
    if name.is_empty() {
        return Err("Name cannot be empty".into());
    }
    if name == "." || name == ".." {
        return Err("Invalid name".into());
    }
    #[cfg(windows)]
    {
        let forbidden = ['<', '>', ':', '"', '/', '\\', '|', '?', '*'];
        if name.chars().any(|c| forbidden.contains(&c)) {
            return Err("Name contains invalid characters".into());
        }
    }
    Ok(())
}

pub fn run() -> iced::Result {
    let boot = persist::load();
    let mut window_settings = window::Settings::default();
    window_settings.fullscreen = boot.window_fullscreen;
    // Do not apply saved dimensions when restoring fullscreen: combined size + fullscreen
    // can create a huge borderless window instead of true fullscreen on Windows.
    if !boot.window_fullscreen {
        if let (Some(w), Some(h)) = (boot.window_width, boot.window_height) {
            if w.is_finite() && h.is_finite() && w >= 320.0 && h >= 240.0 {
                window_settings.size = Size::new(w, h);
            }
        }
    }
    window_settings.icon = crate::window_icon::from_embedded_logo_svg();

    iced::application(App::new, App::update, App::view)
        .subscription(App::subscription)
        .title(App::title)
        .theme(App::theme)
        .font(iced_fonts::LUCIDE_FONT_BYTES)
        .window(window_settings)
        .run()
}
