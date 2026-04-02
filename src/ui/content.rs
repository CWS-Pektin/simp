use iced::highlighter;
use iced::widget::{
    button, column, container, markdown, mouse_area, row, scrollable, space, stack, text,
    text_editor, Id,
};
use iced::{Alignment, Element, Fill, Font, Length};

use crate::app::{App, ContentMode};
use crate::message::Message;
use crate::state::HoverTarget;
use crate::ui::hover_help::hover_track;
use crate::ui::markdown_preview_viewer::MarkdownPreviewViewer;
use crate::ui::theme::{self as t, Colors};

/// [`Id`] for the main markdown preview [`scrollable`] (widget scroll operations target this).
pub const PREVIEW_SCROLLABLE_ID_STR: &str = "markdown-preview-scroll";
/// [`Id`] for the editor’s outer [`scrollable`] (text editor uses [`Length::Shrink`] height inside).
pub const EDITOR_SCROLLABLE_ID_STR: &str = "markdown-editor-scroll";

pub fn view_content(app: &App) -> Element<'_, Message> {
    let c = Colors::for_mode(app.theme_mode);
    let mode = app.theme_mode;
    let mut col = column![].spacing(0);

    if let Some(msg) = &app.conflict_banner {
        col = col.push(
            container(
                row![
                    column![
                        text("⚠  File changed on disk").size(13).color(c.warning),
                        text(msg).size(12).color(c.conflict_text),
                    ]
                    .spacing(2),
                    iced::widget::Space::new().width(Fill),
                    hover_track(
                        button(text("Reload").size(12))
                            .style(move |theme, status| t::btn_primary(theme, status, mode))
                            .padding([5, 10])
                            .on_press(Message::ConflictReloadFromDisk),
                        HoverTarget::ConflictReloadFromDisk,
                    ),
                    hover_track(
                        button(text("Keep local").size(12))
                            .style(move |theme, status| t::btn_ghost(theme, status, mode))
                            .padding([5, 10])
                            .on_press(Message::ConflictKeepLocal),
                        HoverTarget::ConflictKeepLocal,
                    ),
                ]
                .spacing(8)
                .align_y(Alignment::Center),
            )
            .style(move |theme| t::conflict_banner(theme, mode))
            .padding([10, 16])
            .width(Fill),
        );
    }

    col.push(main_pane(app)).into()
}

fn main_pane(app: &App) -> Element<'_, Message> {
    let c = Colors::for_mode(app.theme_mode);
    let mode = app.theme_mode;
    let prev = &app.preferences.preview;
    let ed = &app.preferences.editor;

    if let Some(doc) = &app.current_doc {
        return match app.mode {
            ContentMode::Viewer => hover_track(
                scrollable(
                    container(
                        container({
                            let md_viewer = MarkdownPreviewViewer { mode };
                            markdown::view_with(
                                &doc.markdown_items,
                                t::markdown_settings(
                                    &app.theme,
                                    mode,
                                    prev.base_text_size,
                                ),
                                &md_viewer,
                            )
                        })
                        .max_width(prev.max_content_width as f32)
                        .padding([
                            prev.padding_vertical,
                            prev.padding_horizontal,
                        ]),
                    )
                    .width(Fill)
                    .align_x(Alignment::Center),
                )
                .id(Id::new(PREVIEW_SCROLLABLE_ID_STR))
                .on_scroll(|viewport| {
                    let b = viewport.bounds();
                    let c = viewport.content_bounds();
                    if c.height > b.height + 1.0 {
                        let rel_y = viewport.relative_offset().y;
                        Message::PreviewScrolledRel(if rel_y.is_finite() {
                            Some(rel_y.clamp(0.0, 1.0))
                        } else {
                            None
                        })
                    } else {
                        Message::PreviewScrolledRel(None)
                    }
                })
                .style(move |theme, status| t::content_preview_scrollbar(theme, status, mode))
                .height(Fill)
                .width(Fill),
                HoverTarget::ContentPreviewPane,
            )
            .into(),

            ContentMode::Editor => {
                let hl_theme = if mode == crate::persist::ThemeMode::Dark {
                    highlighter::Theme::Base16Ocean
                } else {
                    highlighter::Theme::InspiredGitHub
                };
                let editor = text_editor(&doc.editor)
                    .placeholder("Start writing Markdown…")
                    .on_action(Message::EditorAction)
                    .height(Length::Shrink)
                    .size(u32::from(ed.font_size))
                    .padding([ed.padding_vertical, ed.padding_horizontal])
                    .font(Font::MONOSPACE)
                    .highlight("markdown", hl_theme);
                // Stack: wheel hits the top [`mouse_area`] first (captures scroll only); clicks fall through to the editor.
                let overlay = mouse_area(space().width(Fill).height(Fill))
                    .on_scroll(Message::EditorOuterScroll);
                hover_track(
                    scrollable(
                        stack![editor, overlay]
                            .width(Fill),
                    )
                    .id(Id::new(EDITOR_SCROLLABLE_ID_STR))
                    .on_scroll(|viewport| {
                        let b = viewport.bounds();
                        let c = viewport.content_bounds();
                        if c.height > b.height + 1.0 {
                            let rel_y = viewport.relative_offset().y;
                            Message::EditorScrolledRel(if rel_y.is_finite() {
                                Some(rel_y.clamp(0.0, 1.0))
                            } else {
                                None
                            })
                        } else {
                            Message::EditorScrolledRel(None)
                        }
                    })
                    .style(move |theme, status| t::content_preview_scrollbar(theme, status, mode))
                    .height(Fill)
                    .width(Fill),
                    HoverTarget::ContentEditorPane,
                )
                .into()
            }
        };
    }

    hover_track(
        container(
            column![
                text("◈").size(40).color(c.toolbar_border),
                iced::widget::Space::new().height(16),
                text("Open a folder to get started").size(16).color(c.status_text),
                iced::widget::Space::new().height(8),
                text("Use the toolbar above to open a project folder or a single Markdown file.")
                    .size(13)
                    .color(c.toolbar_border),
            ]
            .spacing(0)
            .align_x(iced::Alignment::Center),
        )
        .width(Fill)
        .height(Fill)
        .align_x(iced::Alignment::Center)
        .align_y(iced::Alignment::Center),
        HoverTarget::ContentEmptyHint,
    )
    .into()
}
