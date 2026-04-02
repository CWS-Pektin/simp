use iced::padding::Padding;
use iced::widget::markdown::{self, Viewer};
use iced::widget::{column, container, text};
use iced::{Element, Font, Length, Renderer, Theme};

use crate::message::Message;
use crate::persist::ThemeMode;
use crate::ui::theme::Colors;

/// Custom markdown [`Viewer`] that shows the fence language above highlighted code blocks.
#[derive(Clone, Copy)]
pub struct MarkdownPreviewViewer {
    pub mode: ThemeMode,
}

impl<'a> Viewer<'a, Message, Theme, Renderer> for MarkdownPreviewViewer {
    fn on_link_click(url: markdown::Uri) -> Message {
        Message::LinkClicked(url)
    }

    fn code_block(
        &self,
        settings: markdown::Settings,
        language: Option<&'a str>,
        _code: &'a str,
        lines: &'a [markdown::Text],
    ) -> Element<'a, Message, Theme, Renderer> {
        let block = markdown::code_block(settings, lines, Self::on_link_click);

        let lang = language.map(str::trim).filter(|s| !s.is_empty());
        if let Some(lang) = lang {
            let c = Colors::for_mode(self.mode);
            let h_pad = settings.spacing.0 * 0.35 + settings.code_size.0 * 0.2;
            let b_pad = settings.spacing.0 * 0.35;
            column![
                container(
                    text(lang).size(settings.code_size * 0.9).font(Font::MONOSPACE).color(c.sidebar_text_muted),
                )
                .width(Length::Fill)
                .padding(Padding::ZERO.horizontal(h_pad).bottom(b_pad)),
                block,
            ]
            .spacing(0)
            .into()
        } else {
            block
        }
    }
}
