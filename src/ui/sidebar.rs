use std::collections::HashSet;
use std::path::PathBuf;

use iced::alignment;
use iced::widget::{button, column, container, row, scrollable, text};
use iced::{Alignment, Element, Fill, Font, Length};
use iced_fonts::lucide::{file_plus, folder_plus};

use crate::app::App;
use crate::message::Message;
use crate::state::{HoverTarget, NodeKind, Selection, TreeNode};
use crate::ui::hover_help::hover_track;
use crate::ui::theme::{self as t, Colors};

/// Fixed cell for chevron / file bullet so icons align with row text (13px) vertically.
const TREE_GLYPH_COL_W: f32 = 12.0;
const TREE_GLYPH_COL_H: f32 = 18.0;

/// Line box for the project title row; centers the ◈ with the 13px monospace label.
const SIDEBAR_HEADER_LINE_H: f32 = 18.0;

fn project_header_diamond(accent: iced::Color) -> Element<'static, Message> {
    container(
        text("◈")
            .font(Font::MONOSPACE)
            .size(14)
            .color(accent),
    )
    .height(Length::Fixed(SIDEBAR_HEADER_LINE_H))
    .align_y(alignment::Vertical::Center)
    .into()
}

pub fn view_sidebar<'a>(
    app: &'a App,
    tree: Option<&'a TreeNode>,
    expanded: &'a HashSet<PathBuf>,
    selected: Option<&'a Selection>,
) -> Element<'a, Message> {
    let c = Colors::for_mode(app.theme_mode);
    let mode = app.theme_mode;

    let header_text = tree.map(|n| n.name.as_str()).unwrap_or("No project open");

    let header: Element<'_, Message> = if tree.is_some() {
        let new_file = hover_track(
            button(file_plus().size(14))
                .style(move |theme, status| t::btn_ghost(theme, status, mode))
                .padding([4, 6])
                .on_press(Message::NewMarkdownFilePressed),
            HoverTarget::SidebarHeaderNewFile,
        );
        let new_folder = hover_track(
            button(folder_plus().size(14))
                .style(move |theme, status| t::btn_ghost(theme, status, mode))
                .padding([4, 6])
                .on_press(Message::NewFolderPressed),
            HoverTarget::SidebarHeaderNewFolder,
        );
        container(
            row![
                hover_track(
                    row![
                        project_header_diamond(c.accent),
                        iced::widget::Space::new().width(8),
                        container(
                            text(header_text.to_uppercase())
                                .size(13)
                                .font(Font::MONOSPACE)
                                .color(c.sidebar_text),
                        )
                        .height(Length::Fixed(SIDEBAR_HEADER_LINE_H))
                        .align_y(alignment::Vertical::Center),
                    ]
                    .align_y(Alignment::Center),
                    HoverTarget::SidebarProjectHeader,
                ),
                iced::widget::Space::new().width(Fill),
                new_file,
                new_folder,
            ]
            .spacing(4)
            .align_y(Alignment::Center),
        )
        .style(move |theme| t::sidebar_header(theme, mode))
        .width(Fill)
        .padding([10, 14])
        .into()
    } else {
        hover_track(
            container(
                row![
                    project_header_diamond(c.accent),
                    iced::widget::Space::new().width(8),
                    container(
                        text(header_text.to_uppercase())
                            .size(13)
                            .font(Font::MONOSPACE)
                            .color(c.sidebar_text),
                    )
                    .height(Length::Fixed(SIDEBAR_HEADER_LINE_H))
                    .align_y(alignment::Vertical::Center),
                ]
                .align_y(Alignment::Center),
            )
            .style(move |theme| t::sidebar_header(theme, mode))
            .width(Fill)
            .padding([10, 14]),
            HoverTarget::SidebarProjectHeader,
        )
        .into()
    };

    let body: Element<'_, Message> = if let Some(root) = tree {
        let mut col = column![].spacing(1).padding([4, 6]);
        for child in &root.children {
            col = col.push(tree_node_view(child, 0, expanded, selected, mode));
        }
        scrollable(col)
            .style(move |theme, status| t::sidebar_scrollbar(theme, status, mode))
            .height(Fill)
            .into()
    } else {
        scrollable(
            hover_track(
                column![
                    iced::widget::Space::new().height(12),
                    row![
                        iced::widget::Space::new().width(12),
                        text("Open a folder to browse files.")
                            .size(12)
                            .color(c.sidebar_text_muted),
                    ],
                ]
                .spacing(4),
                HoverTarget::SidebarEmptyState,
            ),
        )
        .style(move |theme, status| t::sidebar_scrollbar(theme, status, mode))
        .height(Fill)
        .into()
    };

    column![header, body].into()
}

fn tree_node_view<'a>(
    node: &'a TreeNode,
    depth: u16,
    expanded: &'a HashSet<PathBuf>,
    selected: Option<&'a Selection>,
    mode: crate::persist::ThemeMode,
) -> Element<'a, Message> {
    let c = Colors::for_mode(mode);
    let indent = depth as f32 * 14.0;

    match node.kind {
        NodeKind::MarkdownFile => {
            let is_sel = matches!(selected, Some(Selection::File(p)) if p == &node.path);

            // Match folder rows: same indent + fixed glyph column (chevron width) so root files align with root folders.
            let inner = row![
                iced::widget::Space::new().width(indent),
                container(
                    text("·")
                        .size(11)
                        .color(if is_sel { c.accent } else { c.sidebar_text_muted }),
                )
                .width(Length::Fixed(TREE_GLYPH_COL_W))
                .height(Length::Fixed(TREE_GLYPH_COL_H))
                .align_x(Alignment::Center)
                .align_y(Alignment::Center),
                iced::widget::Space::new().width(5),
                text(&node.name)
                    .size(13)
                    .color(if is_sel { c.sidebar_active_text } else { c.sidebar_text }),
            ]
            .align_y(Alignment::Center);

            hover_track(
                button(inner)
                    .style(move |theme, status| {
                        if is_sel { t::sidebar_item_selected(theme, status, mode) }
                        else { t::sidebar_item(theme, status, mode) }
                    })
                    .width(Fill)
                    .padding([4, 6])
                    .on_press(Message::FileSelected(node.path.clone())),
                HoverTarget::SidebarFile(node.path.clone()),
            )
            .into()
        }

        NodeKind::Category => {
            let is_expanded = expanded.contains(&node.path);
            let is_sel = matches!(selected, Some(Selection::Category(p)) if p == &node.path);

            let chevron = if is_expanded { "▾" } else { "▸" };

            let inner = row![
                iced::widget::Space::new().width(indent),
                container(text(chevron).size(11).color(c.sidebar_text_muted))
                    .width(Length::Fixed(TREE_GLYPH_COL_W))
                    .height(Length::Fixed(TREE_GLYPH_COL_H))
                    .align_x(Alignment::Center)
                    .align_y(Alignment::Center),
                iced::widget::Space::new().width(5),
                text(&node.name)
                    .size(13)
                    .color(if is_sel { c.sidebar_active_text } else { c.sidebar_text }),
            ]
            .align_y(Alignment::Center);

            let path = node.path.clone();
            let header_row = hover_track(
                button(inner)
                    .style(move |theme, status| {
                        if is_sel { t::sidebar_item_selected(theme, status, mode) }
                        else { t::sidebar_item(theme, status, mode) }
                    })
                    .width(Fill)
                    .padding([4, 6])
                    .on_press(Message::FolderRowPressed(node.path.clone())),
                HoverTarget::SidebarFolderRow(path),
            );

            if !is_expanded {
                return column![header_row].spacing(1).into();
            }

            let mut col = column![header_row].spacing(1);
            for child in &node.children {
                col = col.push(tree_node_view(child, depth + 1, expanded, selected, mode));
            }
            col.into()
        }
    }
}
