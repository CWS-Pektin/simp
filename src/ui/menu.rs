use iced::widget::{container, row, text};
use iced::{Alignment, Element, Fill};

use crate::app::App;
use crate::message::Message;
use crate::state::HoverTarget;
use crate::ui::hover_help::{hover_track, workspace_path_line};
use crate::ui::theme::Colors;

pub fn view_menu(app: &App) -> Element<'_, Message> {
    let c = Colors::for_mode(app.theme_mode);

    let path_line = workspace_path_line(app);
    let path_row = hover_track(
        row![text(path_line).size(12).color(c.content_text),]
            .align_y(Alignment::Center),
        HoverTarget::MenuPathContext,
    );

    let inner: Element<'_, Message> = if app.show_settings {
        hover_track(
            row![text("Adjust options in the main area. Use Save to apply theme changes.")
                .size(11)
                .color(c.status_text),]
            .align_y(Alignment::Center),
            HoverTarget::MenuSettingsHint,
        )
        .into()
    } else {
        path_row.into()
    };

    container(inner)
        .padding([4, 16])
        .style(move |_theme| {
            use iced::{Background, Border};
            container::Style {
                background: Some(Background::Color(c.toolbar_bg)),
                border: Border {
                    color: c.toolbar_border,
                    width: 1.0,
                    radius: 0.0.into(),
                },
                ..Default::default()
            }
        })
        .width(Fill)
        .into()
}
