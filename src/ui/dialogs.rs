use iced::widget::{button, column, container, mouse_area, row, stack, text, text_input, Space};
use iced::{Alignment, Element, Fill};

use crate::app::App;
use crate::message::Message;
use crate::state::{DialogState, HoverTarget, SettingsNavTarget};
use crate::ui::hover_help::hover_track;
use crate::ui::theme::{self as t, Colors};

/// Shared size for “New file” and “New folder” modals (larger than content).
const CREATE_DIALOG_WIDTH: f32 = 440.0;
const CREATE_DIALOG_HEIGHT: f32 = 260.0;

/// Centered modal over a dimmed backdrop (create file / folder only).
pub fn view_create_dialog_modal<'a>(app: &'a App, dialog: &'a DialogState) -> Element<'a, Message> {
    let mode = app.theme_mode;
    stack![
        mouse_area(
            container(Space::new().width(Fill).height(Fill))
                .style(move |theme| t::dialog_backdrop(theme, mode)),
        )
        .on_press(Message::CancelCreateDialog),
        container(
            container(view_dialog_overlay(app, dialog))
                .style(move |theme| t::dialog_card(theme, mode))
                .width(CREATE_DIALOG_WIDTH)
                .height(CREATE_DIALOG_HEIGHT),
        )
        .center(Fill),
    ]
    .width(Fill)
    .height(Fill)
    .into()
}

pub fn view_dialog_overlay<'a>(app: &'a App, dialog: &'a DialogState) -> Element<'a, Message> {
    let c = Colors::for_mode(app.theme_mode);
    let mode = app.theme_mode;

    if let DialogState::UnsavedSettings { target } = dialog {
        let (title, body) = match target {
            SettingsNavTarget::Category(_) => (
                "Unsaved changes",
                "Save or discard your settings before switching to another category.",
            ),
            SettingsNavTarget::ExitSettings => (
                "Unsaved changes",
                "Save or discard your settings before closing this panel.",
            ),
        };
        return column![
            row![
                text("◆").size(16).color(c.warning),
                iced::widget::Space::new().width(8),
                text(title).size(15).color(c.content_text),
            ]
            .align_y(Alignment::Center),
            iced::widget::Space::new().height(10),
            text(body).size(12).color(c.status_text),
            iced::widget::Space::new().height(20),
            row![
                hover_track(
                    button(text("Save").size(13))
                        .style(move |theme, status| t::btn_primary(theme, status, mode))
                        .padding([7, 14])
                        .on_press(Message::UnsavedSettingsSave { target: *target }),
                    HoverTarget::DialogUnsavedSave,
                ),
                hover_track(
                    button(text("Discard").size(13))
                        .style(move |theme, status| t::btn_ghost(theme, status, mode))
                        .padding([7, 12])
                        .on_press(Message::UnsavedSettingsDiscard { target: *target }),
                    HoverTarget::DialogUnsavedDiscard,
                ),
                hover_track(
                    button(text("Close").size(13))
                        .style(move |theme, status| t::btn_ghost(theme, status, mode))
                        .padding([7, 12])
                        .on_press(Message::UnsavedSettingsClose),
                    HoverTarget::DialogUnsavedClose,
                ),
            ]
            .spacing(8),
        ]
        .spacing(0)
        .padding(24)
        .into();
    }

    let (title, icon, hint) = match dialog {
        DialogState::NewMarkdownFile { .. } => (
            "New Markdown File",
            "◻",
            "Filename — .md will be added automatically",
        ),
        DialogState::NewFolder { .. } => ("New Folder", "▸", "Folder name"),
        DialogState::UnsavedSettings { .. } => unreachable!(),
    };

    let input_val = match dialog {
        DialogState::NewMarkdownFile { input, .. } => input.as_str(),
        DialogState::NewFolder { input, .. } => input.as_str(),
        DialogState::UnsavedSettings { .. } => "",
    };

    column![
        row![
            text(icon).size(16).color(c.accent),
            iced::widget::Space::new().width(8),
            text(title).size(15).color(c.content_text),
        ]
        .align_y(Alignment::Center),
        iced::widget::Space::new().height(4),
        text(hint).size(11).color(c.status_text),
        iced::widget::Space::new().height(14),
        hover_track(
            text_input("Enter name…", input_val)
                .on_input(Message::CreateDialogInputChanged)
                .on_submit(Message::ConfirmCreateDialog)
                .style(move |theme, status| t::input_style(theme, status, mode))
                .width(Fill)
                .padding([8, 10])
                .size(13),
            HoverTarget::DialogCreateNameInput,
        ),
        Space::new().height(Fill),
        row![
            hover_track(
                button(text("Create").size(13))
                    .style(move |theme, status| t::btn_primary(theme, status, mode))
                    .padding([7, 16])
                    .on_press(Message::ConfirmCreateDialog),
                HoverTarget::DialogCreateSubmit,
            ),
            hover_track(
                button(text("Cancel").size(13))
                    .style(move |theme, status| t::btn_ghost(theme, status, mode))
                    .padding([7, 12])
                    .on_press(Message::CancelCreateDialog),
                HoverTarget::DialogCreateCancel,
            ),
        ]
        .spacing(8),
    ]
    .spacing(0)
    .width(Fill)
    .height(Fill)
    .padding(24)
    .into()
}
