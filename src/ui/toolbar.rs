use iced::widget::{button, row, text, toggler};
use iced::{Alignment, Element, Fill};
use iced_fonts::lucide::{file, folder_open, settings as settings_icon};

use crate::app::{App, ContentMode};
use crate::message::Message;
use crate::state::HoverTarget;
use crate::ui::hover_help::hover_track;
use crate::ui::theme::{self as t, Colors};

pub fn view_toolbar(app: &App) -> Element<'_, Message> {
    let c = Colors::for_mode(app.theme_mode);

    let settings_btn = hover_track(
        button(settings_icon().size(16))
            .style(move |theme, status| {
                if app.show_settings {
                    t::sidebar_item_selected(theme, status, app.theme_mode)
                } else {
                    t::btn_ghost(theme, status, app.theme_mode)
                }
            })
            .padding([6, 10])
            .on_press(Message::SettingsPressed),
        HoverTarget::SettingsButton,
    );

    let open_folder = hover_track(
        button(folder_open().size(16))
            .style(move |theme, status| t::btn_primary(theme, status, app.theme_mode))
            .padding([6, 10])
            .on_press(Message::OpenFolderPressed),
        HoverTarget::OpenFolderButton,
    );

    let open_file = hover_track(
        button(file().size(16))
            .style(move |theme, status| t::btn_ghost(theme, status, app.theme_mode))
            .padding([6, 10])
            .on_press(Message::OpenFilePressed),
        HoverTarget::OpenFileButton,
    );

    let open_actions: Element<'_, Message> = if app.show_settings {
        row![].into()
    } else {
        row![
            iced::widget::Space::new().width(4),
            open_folder,
            open_file,
        ]
        .spacing(8)
        .align_y(Alignment::Center)
        .into()
    };

    let mode_toggle: Element<'_, Message> = if app.current_doc.is_some() && !app.show_settings {
        hover_track(
            row![
                text("Editor").size(12).color(c.status_text),
                toggler(matches!(app.mode, ContentMode::Viewer))
                    .on_toggle(|v| if v { Message::ModeViewer } else { Message::ModeEditor }),
                text("Preview").size(12).color(c.status_text),
            ]
            .spacing(6)
            .align_y(Alignment::Center),
            HoverTarget::EditorPreviewToggle,
        )
        .into()
    } else {
        text("").size(1).into()
    };

    row![
        settings_btn,
        open_actions,
        iced::widget::Space::new().width(Fill),
        mode_toggle,
    ]
    .spacing(8)
    .padding([8, 16])
    .align_y(Alignment::Center)
    .into()
}
