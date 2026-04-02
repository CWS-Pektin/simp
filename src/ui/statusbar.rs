use iced::widget::{row, text};
use iced::{Alignment, Element, Fill};

use crate::app::App;
use crate::message::Message;
use crate::state::{HoverTarget, StatusState};
use crate::ui::hover_help::hover_track;
use crate::ui::theme::Colors;

pub fn view_statusbar(app: &App) -> Element<'_, Message> {
    let c = Colors::for_mode(app.theme_mode);

    let help = if app.preferences.general.show_status_hover_hints {
        app.hover_target
            .as_ref()
            .map(|t| t.help_line())
            .unwrap_or_else(|| "—".into())
    } else {
        "—".into()
    };

    let document_status: Element<'_, Message> = {
        let inner: Element<'_, Message> = match &app.status {
            StatusState::Saving => row![
                text("●").size(9).color(c.warning),
                text("  Saving…").size(11).color(c.status_text),
            ]
            .align_y(Alignment::Center)
            .into(),
            StatusState::Saved => row![
                text("●").size(9).color(c.success),
                text("  Saved").size(11).color(c.status_text),
            ]
            .align_y(Alignment::Center)
            .into(),
            StatusState::Error(e) => row![
                text("●").size(9).color(c.danger),
                text(format!("  {e}")).size(11).color(c.danger),
            ]
            .align_y(Alignment::Center)
            .into(),
            StatusState::Conflict(msg) => row![
                text("●").size(9).color(c.warning),
                text(format!("  {msg}")).size(11).color(c.warning),
            ]
            .align_y(Alignment::Center)
            .into(),
            StatusState::Idle => {
                let show_no_changes = app
                    .current_doc
                    .as_ref()
                    .is_some_and(|d| !d.dirty && !d.save_pending);
                if show_no_changes {
                    text("No changes yet").size(11).color(c.status_text).into()
                } else {
                    text("").size(1).into()
                }
            }
        };
        hover_track(inner, HoverTarget::StatusDocument).into()
    };

    row![
        text(help).size(11).color(c.status_text),
        iced::widget::Space::new().width(Fill),
        document_status,
        iced::widget::Space::new().width(12),
    ]
    .align_y(Alignment::Center)
    .padding([4, 16])
    .into()
}
