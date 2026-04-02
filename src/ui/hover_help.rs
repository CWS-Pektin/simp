use iced::widget::mouse_area;
use iced::Element;

use crate::app::App;
use crate::message::Message;
use crate::state::{HoverTarget, Selection};

pub fn hover_track<'a>(
    inner: impl Into<Element<'a, Message>>,
    target: HoverTarget,
) -> Element<'a, Message> {
    let enter = target.clone();
    mouse_area(inner.into())
        .on_enter(Message::HoverEnter(enter))
        .on_exit(Message::HoverLeave(target))
        .into()
}

/// Same path semantics as the former status bar: open document, then selection, else em dash.
pub fn workspace_path_line(app: &App) -> String {
    app.current_doc
        .as_ref()
        .map(|d| d.path.display().to_string())
        .or_else(|| {
            app.selected.as_ref().map(|s| match s {
                Selection::Category(p) => p.display().to_string(),
                Selection::File(p) => p.display().to_string(),
            })
        })
        .unwrap_or_else(|| "—".into())
}
