use iced::alignment;
use iced::widget::{button, column, container, row, scrollable, text};
use iced::{Alignment, Element, Fill, Font, Length};
use iced_fonts::lucide::settings as settings_icon;

use crate::app::App;
use crate::message::Message;
use crate::state::{HoverTarget, SettingsCategory};
use crate::ui::hover_help::hover_track;
use crate::ui::theme::{self as t, Colors};

/// Same line box as `sidebar::project_header` so SETTINGS matches LABELDESK-DOCS height and alignment.
const SIDEBAR_HEADER_LINE_H: f32 = 18.0;

pub fn view_settings_sidebar(app: &App) -> Element<'_, Message> {
    let c = Colors::for_mode(app.theme_mode);
    let mode = app.theme_mode;

    let header = hover_track(
        container(
            row![
                container(settings_icon().size(14).color(c.accent))
                    .height(Length::Fixed(SIDEBAR_HEADER_LINE_H))
                    .align_y(alignment::Vertical::Center),
                iced::widget::Space::new().width(8),
                container(
                    text("SETTINGS")
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
        HoverTarget::SettingsSidebarHeader,
    );

    let mut col = column![].spacing(1).padding([4, 6]);
    for cat in SettingsCategory::ALL {
        let is_sel = app.settings_category == cat;
        col = col.push(hover_track(
            button(text(cat.label()).size(13).color(if is_sel {
                c.sidebar_active_text
            } else {
                c.sidebar_text
            }))
            .style(move |theme, status| {
                if is_sel {
                    t::sidebar_item_selected(theme, status, mode)
                } else {
                    t::sidebar_item(theme, status, mode)
                }
            })
            .width(Fill)
            .padding([6, 10])
            .on_press(Message::SettingsCategorySelected(cat)),
            HoverTarget::SettingsSidebarCategory(cat),
        ));
    }

    let body = scrollable(col)
        .style(move |theme, status| t::sidebar_scrollbar(theme, status, mode))
        .height(Fill);

    column![header, body].into()
}
