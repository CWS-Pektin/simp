/// Visual design system with dark and light mode support.
use iced::{border, padding, Background, Border, Color, Shadow, Theme, Vector};
use iced::theme::Palette;
use iced::widget::{button, container, markdown, scrollable, text_input};

use crate::persist::ThemeMode;

// ── RGB helper ────────────────────────────────────────────────────────────────

const fn rgb(r: u8, g: u8, b: u8) -> Color {
    Color { r: r as f32 / 255.0, g: g as f32 / 255.0, b: b as f32 / 255.0, a: 1.0 }
}

// ── Light mode palette ────────────────────────────────────────────────────────

pub mod light {
    use super::*;

    pub const SIDEBAR_BG: Color          = rgb(0xF5, 0xF7, 0xFA); // #F5F7FA
    pub const SIDEBAR_HEADER_BG: Color   = rgb(0xE8, 0xEB, 0xF0); // #E8EBF0
    pub const SIDEBAR_TEXT: Color        = rgb(0x33, 0x3F, 0x52); // #333F52
    pub const SIDEBAR_TEXT_MUTED: Color  = rgb(0x6B, 0x7A, 0x8A); // #6B7A8A
    pub const SIDEBAR_ACTIVE_BG: Color   = rgb(0x4C, 0x8B, 0xF5); // #4C8BF5
    pub const SIDEBAR_ACTIVE_TEXT: Color = Color::WHITE;
    pub const SIDEBAR_HOVER_BG: Color    = rgb(0xE0, 0xE4, 0xEA); // #E0E4EA
    pub const SIDEBAR_BORDER: Color      = rgb(0xD4, 0xD9, 0xE1); // #D4D9E1

    pub const TOOLBAR_BG: Color     = Color::WHITE;
    pub const TOOLBAR_BORDER: Color = rgb(0xE0, 0xE6, 0xED); // #E0E6ED

    pub const CONTENT_BG: Color   = Color::WHITE;
    pub const CONTENT_TEXT: Color = rgb(0x15, 0x1A, 0x24); // #151A24

    pub const STATUS_BG: Color     = rgb(0xF4, 0xF6, 0xF9); // #F4F6F9
    pub const STATUS_BORDER: Color = rgb(0xD9, 0xDF, 0xE8); // #D9DFE8
    pub const STATUS_TEXT: Color   = rgb(0x6B, 0x7A, 0x8A); // #6B7A8A

    pub const ACCENT: Color  = rgb(0x4C, 0x8B, 0xF5); // #4C8BF5
    pub const SUCCESS: Color = rgb(0x34, 0xB4, 0x75); // #34B475
    pub const DANGER: Color  = rgb(0xEB, 0x53, 0x5A); // #EB535A
    pub const WARNING: Color = rgb(0xF4, 0xA6, 0x1F); // #F4A61F

    pub const CONFLICT_BG: Color     = rgb(0xFF, 0xF5, 0xEB); // #FFF5EB
    pub const CONFLICT_TEXT: Color   = rgb(0x80, 0x4F, 0x00); // #804F00
    pub const CONFLICT_BORDER: Color = rgb(0xED, 0xB8, 0x51); // #EDB851

    /// Inline `code` in markdown preview (Iced default is near-black + white text).
    pub const MD_INLINE_CODE_BG: Color = rgb(0xF0, 0xF2, 0xF5); // #F0F2F5
    pub const MD_INLINE_CODE_FG: Color = rgb(0x2D, 0x33, 0x3D); // #2D333D
}

// ── Dark mode palette ─────────────────────────────────────────────────────────

pub mod dark {
    use super::*;

    pub const SIDEBAR_BG: Color          = rgb(0x1B, 0x22, 0x30); // #1B2230
    pub const SIDEBAR_HEADER_BG: Color   = rgb(0x13, 0x19, 0x22); // #131922
    pub const SIDEBAR_TEXT: Color        = rgb(0xB2, 0xBF, 0xCF); // #B2BFCF
    pub const SIDEBAR_TEXT_MUTED: Color  = rgb(0x6B, 0x7A, 0x8A); // #6B7A8A
    pub const SIDEBAR_ACTIVE_BG: Color   = rgb(0x28, 0x51, 0xA0); // #2851A0
    pub const SIDEBAR_ACTIVE_TEXT: Color = rgb(0xF0, 0xF5, 0xFF); // #F0F5FF
    pub const SIDEBAR_HOVER_BG: Color    = rgb(0x24, 0x2D, 0x39); // #242D39
    pub const SIDEBAR_BORDER: Color      = rgb(0x10, 0x15, 0x1C); // #10151C

    pub const TOOLBAR_BG: Color     = rgb(0x1E, 0x25, 0x33); // #1E2533
    pub const TOOLBAR_BORDER: Color = rgb(0x2A, 0x33, 0x42); // #2A3342

    pub const CONTENT_BG: Color   = rgb(0x23, 0x2A, 0x38); // #232A38
    pub const CONTENT_TEXT: Color = rgb(0xE4, 0xE7, 0xEB); // #E4E7EB

    pub const STATUS_BG: Color     = rgb(0x18, 0x1E, 0x2A); // #181E2A
    pub const STATUS_BORDER: Color = rgb(0x2D, 0x36, 0x45); // #2D3645
    pub const STATUS_TEXT: Color   = rgb(0x8B, 0x98, 0xA8); // #8B98A8

    pub const ACCENT: Color  = rgb(0x5A, 0x9C, 0xFF); // #5A9CFF
    pub const SUCCESS: Color = rgb(0x4A, 0xD4, 0x8F); // #4AD48F
    pub const DANGER: Color  = rgb(0xFF, 0x6B, 0x72); // #FF6B72
    pub const WARNING: Color = rgb(0xFF, 0xC1, 0x5A); // #FFC15A

    pub const CONFLICT_BG: Color     = rgb(0x3D, 0x2E, 0x1A); // #3D2E1A
    pub const CONFLICT_TEXT: Color   = rgb(0xFF, 0xC1, 0x5A); // #FFC15A
    pub const CONFLICT_BORDER: Color = rgb(0x8B, 0x6B, 0x3A); // #8B6B3A

    pub const MD_INLINE_CODE_BG: Color = rgb(0x35, 0x3D, 0x4D); // #353D4D
    pub const MD_INLINE_CODE_FG: Color = rgb(0xE8, 0xEC, 0xF2); // #E8ECF2
}

// ── Dynamic color resolver ────────────────────────────────────────────────────

pub struct Colors {
    pub sidebar_bg: Color,
    pub sidebar_header_bg: Color,
    pub sidebar_text: Color,
    pub sidebar_text_muted: Color,
    pub sidebar_active_bg: Color,
    pub sidebar_active_text: Color,
    pub sidebar_hover_bg: Color,
    pub sidebar_border: Color,
    pub toolbar_bg: Color,
    pub toolbar_border: Color,
    pub content_bg: Color,
    pub content_text: Color,
    pub status_bg: Color,
    pub status_border: Color,
    pub status_text: Color,
    pub accent: Color,
    pub success: Color,
    pub danger: Color,
    pub warning: Color,
    pub conflict_bg: Color,
    pub conflict_text: Color,
    pub conflict_border: Color,
    pub md_inline_code_bg: Color,
    pub md_inline_code_fg: Color,
}

impl Colors {
    pub fn for_mode(mode: ThemeMode) -> Self {
        match mode {
            ThemeMode::Light => Self {
                sidebar_bg: light::SIDEBAR_BG,
                sidebar_header_bg: light::SIDEBAR_HEADER_BG,
                sidebar_text: light::SIDEBAR_TEXT,
                sidebar_text_muted: light::SIDEBAR_TEXT_MUTED,
                sidebar_active_bg: light::SIDEBAR_ACTIVE_BG,
                sidebar_active_text: light::SIDEBAR_ACTIVE_TEXT,
                sidebar_hover_bg: light::SIDEBAR_HOVER_BG,
                sidebar_border: light::SIDEBAR_BORDER,
                toolbar_bg: light::TOOLBAR_BG,
                toolbar_border: light::TOOLBAR_BORDER,
                content_bg: light::CONTENT_BG,
                content_text: light::CONTENT_TEXT,
                status_bg: light::STATUS_BG,
                status_border: light::STATUS_BORDER,
                status_text: light::STATUS_TEXT,
                accent: light::ACCENT,
                success: light::SUCCESS,
                danger: light::DANGER,
                warning: light::WARNING,
                conflict_bg: light::CONFLICT_BG,
                conflict_text: light::CONFLICT_TEXT,
                conflict_border: light::CONFLICT_BORDER,
                md_inline_code_bg: light::MD_INLINE_CODE_BG,
                md_inline_code_fg: light::MD_INLINE_CODE_FG,
            },
            ThemeMode::Dark => Self {
                sidebar_bg: dark::SIDEBAR_BG,
                sidebar_header_bg: dark::SIDEBAR_HEADER_BG,
                sidebar_text: dark::SIDEBAR_TEXT,
                sidebar_text_muted: dark::SIDEBAR_TEXT_MUTED,
                sidebar_active_bg: dark::SIDEBAR_ACTIVE_BG,
                sidebar_active_text: dark::SIDEBAR_ACTIVE_TEXT,
                sidebar_hover_bg: dark::SIDEBAR_HOVER_BG,
                sidebar_border: dark::SIDEBAR_BORDER,
                toolbar_bg: dark::TOOLBAR_BG,
                toolbar_border: dark::TOOLBAR_BORDER,
                content_bg: dark::CONTENT_BG,
                content_text: dark::CONTENT_TEXT,
                status_bg: dark::STATUS_BG,
                status_border: dark::STATUS_BORDER,
                status_text: dark::STATUS_TEXT,
                accent: dark::ACCENT,
                success: dark::SUCCESS,
                danger: dark::DANGER,
                warning: dark::WARNING,
                conflict_bg: dark::CONFLICT_BG,
                conflict_text: dark::CONFLICT_TEXT,
                conflict_border: dark::CONFLICT_BORDER,
                md_inline_code_bg: dark::MD_INLINE_CODE_BG,
                md_inline_code_fg: dark::MD_INLINE_CODE_FG,
            },
        }
    }
}

pub fn build_theme(mode: ThemeMode) -> Theme {
    let c = Colors::for_mode(mode);
    Theme::custom("Docs", Palette {
        background: c.content_bg,
        text: c.content_text,
        primary: c.accent,
        success: c.success,
        warning: c.warning,
        danger: c.danger,
    })
}

/// Markdown preview settings: Iced's default uses `#111` + white for inline code.
///
/// `base_text_size` is in logical pixels (px), matching persisted preview preferences.
pub fn markdown_settings(theme: &Theme, mode: ThemeMode, base_text_size: u16) -> markdown::Settings {
    let mut style = markdown::Style::from(theme);
    let c = Colors::for_mode(mode);
    style.inline_code_highlight = markdown::Highlight {
        background: Background::Color(c.md_inline_code_bg),
        border: border::rounded(4),
    };
    style.inline_code_color = c.md_inline_code_fg;
    // Keep horizontal padding small: in `rich_text`, a wide highlight draws past the
    // glyph box and can sit on top of the normal space before/after the span, so
    // gaps look missing or inconsistent. Vertical padding is safer for line height.
    style.inline_code_padding = padding::left(2).right(2).top(2).bottom(2);
    markdown::Settings::with_text_size(u32::from(base_text_size), style)
}

// ── Container styles ──────────────────────────────────────────────────────────

pub fn sidebar_pane(_theme: &Theme, mode: ThemeMode) -> container::Style {
    let c = Colors::for_mode(mode);
    container::Style {
        background: Some(Background::Color(c.sidebar_bg)),
        text_color: Some(c.sidebar_text),
        ..Default::default()
    }
}

pub fn sidebar_header(_theme: &Theme, mode: ThemeMode) -> container::Style {
    let c = Colors::for_mode(mode);
    container::Style {
        background: Some(Background::Color(c.sidebar_header_bg)),
        text_color: Some(c.sidebar_text_muted),
        border: Border {
            color: c.sidebar_border,
            width: 1.0,
            radius: 0.0.into(),
        },
        ..Default::default()
    }
}

pub fn toolbar_bar(_theme: &Theme, mode: ThemeMode) -> container::Style {
    let c = Colors::for_mode(mode);
    container::Style {
        background: Some(Background::Color(c.toolbar_bg)),
        shadow: Shadow {
            color: Color { r: 0.0, g: 0.0, b: 0.0, a: if mode == ThemeMode::Dark { 0.3 } else { 0.07 } },
            offset: Vector::new(0.0, 1.5),
            blur_radius: 4.0,
        },
        ..Default::default()
    }
}

pub fn content_pane(_theme: &Theme, mode: ThemeMode) -> container::Style {
    let c = Colors::for_mode(mode);
    container::Style {
        background: Some(Background::Color(c.content_bg)),
        text_color: Some(c.content_text),
        ..Default::default()
    }
}

pub fn status_bar(_theme: &Theme, mode: ThemeMode) -> container::Style {
    let c = Colors::for_mode(mode);
    container::Style {
        background: Some(Background::Color(c.status_bg)),
        text_color: Some(c.status_text),
        border: Border {
            color: c.status_border,
            width: 1.0,
            radius: 0.0.into(),
        },
        ..Default::default()
    }
}

pub fn dialog_backdrop(_theme: &Theme, _mode: ThemeMode) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 0.42,
        })),
        ..Default::default()
    }
}

pub fn dialog_card(_theme: &Theme, mode: ThemeMode) -> container::Style {
    let c = Colors::for_mode(mode);
    container::Style {
        background: Some(Background::Color(if mode == ThemeMode::Dark {
            rgb(0x2A, 0x32, 0x42)
        } else {
            Color::WHITE
        })),
        text_color: Some(c.content_text),
        border: Border {
            color: if mode == ThemeMode::Dark {
                rgb(0x3A, 0x44, 0x56)
            } else {
                rgb(0xD4, 0xDC, 0xE6)
            },
            width: 1.0,
            radius: 8.0.into(),
        },
        shadow: Shadow {
            color: Color { r: 0.0, g: 0.0, b: 0.0, a: if mode == ThemeMode::Dark { 0.5 } else { 0.18 } },
            offset: Vector::new(0.0, 4.0),
            blur_radius: 16.0,
        },
        ..Default::default()
    }
}

pub fn conflict_banner(_theme: &Theme, mode: ThemeMode) -> container::Style {
    let c = Colors::for_mode(mode);
    container::Style {
        background: Some(Background::Color(c.conflict_bg)),
        text_color: Some(c.conflict_text),
        border: Border {
            color: c.conflict_border,
            width: 1.0,
            radius: 0.0.into(),
        },
        ..Default::default()
    }
}

// ── Button styles ─────────────────────────────────────────────────────────────

pub fn btn_primary(_theme: &Theme, status: button::Status, mode: ThemeMode) -> button::Style {
    let c = Colors::for_mode(mode);
    button::Style {
        background: Some(Background::Color(match status {
            button::Status::Disabled => {
                if mode == ThemeMode::Dark {
                    rgb(0x38, 0x44, 0x52)
                } else {
                    rgb(0xC5, 0xD0, 0xDD)
                }
            }
            button::Status::Hovered => rgb(0x32, 0x74, 0xE0),
            button::Status::Pressed => rgb(0x26, 0x60, 0xC8),
            _ => c.accent,
        })),
        text_color: match status {
            button::Status::Disabled => {
                if mode == ThemeMode::Dark {
                    rgb(0x7A, 0x88, 0x9A)
                } else {
                    rgb(0x68, 0x74, 0x84)
                }
            }
            _ => Color::WHITE,
        },
        border: Border { radius: 6.0.into(), ..Default::default() },
        ..Default::default()
    }
}

pub fn btn_ghost(_theme: &Theme, status: button::Status, mode: ThemeMode) -> button::Style {
    let c = Colors::for_mode(mode);
    button::Style {
        background: Some(Background::Color(match (status, mode) {
            (button::Status::Hovered, ThemeMode::Light)  => rgb(0xF0, 0xF3, 0xF7),
            (button::Status::Pressed, ThemeMode::Light)  => rgb(0xE0, 0xE6, 0xED),
            (button::Status::Hovered, ThemeMode::Dark)   => rgb(0x2D, 0x36, 0x45),
            (button::Status::Pressed, ThemeMode::Dark)   => rgb(0x24, 0x2C, 0x39),
            (_, ThemeMode::Light)                        => Color::WHITE,
            (_, ThemeMode::Dark)                         => c.toolbar_bg,
        })),
        text_color: c.content_text,
        border: Border {
            color: if mode == ThemeMode::Dark { rgb(0x3A, 0x44, 0x56) } else { rgb(0xC7, 0xD1, 0xDE) },
            width: 1.0,
            radius: 6.0.into(),
        },
        ..Default::default()
    }
}

pub fn sidebar_item(_theme: &Theme, status: button::Status, mode: ThemeMode) -> button::Style {
    let c = Colors::for_mode(mode);
    button::Style {
        background: Some(Background::Color(match status {
            button::Status::Hovered | button::Status::Pressed => c.sidebar_hover_bg,
            _ => Color::TRANSPARENT,
        })),
        text_color: c.sidebar_text,
        border: Border { radius: 4.0.into(), ..Default::default() },
        ..Default::default()
    }
}

pub fn sidebar_item_selected(_theme: &Theme, status: button::Status, mode: ThemeMode) -> button::Style {
    let c = Colors::for_mode(mode);
    button::Style {
        background: Some(Background::Color(match status {
            button::Status::Pressed => if mode == ThemeMode::Dark {
                rgb(0x20, 0x48, 0x91)
            } else {
                rgb(0x3A, 0x72, 0xD8)
            },
            _ => c.sidebar_active_bg,
        })),
        text_color: c.sidebar_active_text,
        border: Border { radius: 4.0.into(), ..Default::default() },
        ..Default::default()
    }
}

// ── Input style ───────────────────────────────────────────────────────────────

pub fn input_style(theme: &Theme, status: text_input::Status, mode: ThemeMode) -> text_input::Style {
    let c = Colors::for_mode(mode);
    let base = text_input::default(theme, status);
    text_input::Style {
        background: Background::Color(if mode == ThemeMode::Dark {
            rgb(0x2A, 0x32, 0x42)
        } else {
            Color::WHITE
        }),
        border: Border {
            color: match status {
                text_input::Status::Focused { .. } => c.accent,
                _ => if mode == ThemeMode::Dark { rgb(0x3A, 0x44, 0x56) } else { rgb(0xD4, 0xDC, 0xE6) },
            },
            width: 1.5,
            radius: 6.0.into(),
        },
        ..base
    }
}

// ── Scrollable ────────────────────────────────────────────────────────────────

pub fn sidebar_scrollbar(theme: &Theme, status: scrollable::Status, mode: ThemeMode) -> scrollable::Style {
    let c = Colors::for_mode(mode);
    let base = scrollable::default(theme, status);
    scrollable::Style {
        container: container::Style {
            background: Some(Background::Color(c.sidebar_bg)),
            text_color: Some(c.sidebar_text),
            ..base.container
        },
        vertical_rail: scrollable::Rail {
            background: None,
            border: Border { radius: 4.0.into(), ..Default::default() },
            scroller: scrollable::Scroller {
                background: Background::Color(if mode == ThemeMode::Dark {
                    rgb(0x38, 0x48, 0x5E)
                } else {
                    rgb(0xC0, 0xCA, 0xD8)
                }),
                border: Border { radius: 4.0.into(), ..Default::default() },
            },
        },
        ..base
    }
}

pub fn content_preview_scrollbar(theme: &Theme, status: scrollable::Status, mode: ThemeMode) -> scrollable::Style {
    let c = Colors::for_mode(mode);
    let base = scrollable::default(theme, status);
    scrollable::Style {
        container: container::Style {
            background: Some(Background::Color(c.content_bg)),
            text_color: Some(c.content_text),
            ..base.container
        },
        vertical_rail: scrollable::Rail {
            background: None,
            border: Border { radius: 4.0.into(), ..Default::default() },
            scroller: scrollable::Scroller {
                background: Background::Color(if mode == ThemeMode::Dark {
                    rgb(0x38, 0x48, 0x5E)
                } else {
                    rgb(0xC0, 0xCA, 0xD8)
                }),
                border: Border { radius: 4.0.into(), ..Default::default() },
            },
        },
        ..base
    }
}
