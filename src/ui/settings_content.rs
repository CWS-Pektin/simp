use std::fmt;

use iced::widget::{button, column, container, pick_list, row, text, toggler};
use iced::{Alignment, Element, Fill};

use crate::app::App;
use crate::message::{Message, SettingsPreferenceChanged};
use crate::persist::ThemeMode;
use crate::state::{HoverTarget, SettingsCategory};
use crate::ui::hover_help::hover_track;
use crate::ui::theme::{self as t, Colors};

const DEBOUNCE_MS: &[u32] = &[250, 500, 1000, 2000];
const PREVIEW_WIDTHS: &[u32] = &[720, 820, 960, 1120];
const PREVIEW_TEXT_SIZES: &[u16] = &[14, 15, 16, 17, 18];
const PREVIEW_PADS: &[u16] = &[24, 32, 40, 48];
const EDITOR_FONTS: &[u16] = &[12, 13, 14, 15, 16];
const EDITOR_PAD_H: &[u16] = &[12, 16, 20, 24, 32];
const EDITOR_PAD_V: &[u16] = &[8, 12, 16, 20, 24];

/// Pick-list row label for values shown as `14px` in the dropdown (not in the row text).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PxU16(u16);

impl fmt::Display for PxU16 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}px", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PxU32(u32);

impl fmt::Display for PxU32 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}px", self.0)
    }
}

fn px_u16_options(values: &[u16]) -> Vec<PxU16> {
    values.iter().copied().map(PxU16).collect()
}

fn px_u32_options(values: &[u32]) -> Vec<PxU32> {
    values.iter().copied().map(PxU32).collect()
}

pub fn view_settings_content(app: &App) -> Element<'_, Message> {
    let c = Colors::for_mode(app.theme_mode);
    let mode = app.theme_mode;
    let d = &app.preferences_draft;

    let panel: Element<'_, Message> = match app.settings_category {
        SettingsCategory::General => column![
            text("General").size(22).color(c.content_text),
            iced::widget::Space::new().height(16),
            row![
                text("Auto-save delay after last edit").size(13).color(c.status_text),
                iced::widget::Space::new().width(Fill),
                hover_track(
                    pick_list(
                        DEBOUNCE_MS.to_vec(),
                        Some(d.general.auto_save_debounce_ms),
                        |ms| {
                            Message::SettingsPreferenceChanged(
                                SettingsPreferenceChanged::AutoSaveDebounceMs(ms),
                            )
                        },
                    )
                    .placeholder("ms")
                    .text_size(13)
                    .padding([6, 10]),
                    HoverTarget::SettingsAutoSaveDebounce,
                ),
            ]
            .spacing(12)
            .align_y(Alignment::Center),
            iced::widget::Space::new().height(12),
            row![
                text("Show status-bar hover hints").size(13).color(c.status_text),
                iced::widget::Space::new().width(Fill),
                hover_track(
                    toggler(d.general.show_status_hover_hints).on_toggle(|v| {
                        Message::SettingsPreferenceChanged(
                            SettingsPreferenceChanged::ShowStatusHoverHints(v),
                        )
                    }),
                    HoverTarget::SettingsShowStatusHints,
                ),
            ]
            .spacing(12)
            .align_y(Alignment::Center),
        ]
        .spacing(0)
        .into(),
        SettingsCategory::Preview => column![
            text("Preview").size(22).color(c.content_text),
            iced::widget::Space::new().height(16),
            row![
                text("Max content width").size(13).color(c.status_text),
                iced::widget::Space::new().width(Fill),
                hover_track(
                    pick_list(
                        px_u32_options(PREVIEW_WIDTHS),
                        Some(PxU32(d.preview.max_content_width)),
                        |PxU32(w)| {
                            Message::SettingsPreferenceChanged(
                                SettingsPreferenceChanged::PreviewMaxWidth(w),
                            )
                        },
                    )
                    .placeholder("—")
                    .text_size(13)
                    .padding([6, 10]),
                    HoverTarget::SettingsPreviewMaxWidth,
                ),
            ]
            .spacing(12)
            .align_y(Alignment::Center),
            iced::widget::Space::new().height(12),
            row![
                text("Preview font size").size(13).color(c.status_text),
                iced::widget::Space::new().width(Fill),
                hover_track(
                    pick_list(
                        px_u16_options(PREVIEW_TEXT_SIZES),
                        Some(PxU16(d.preview.base_text_size)),
                        |PxU16(s)| {
                            Message::SettingsPreferenceChanged(
                                SettingsPreferenceChanged::PreviewBaseTextSize(s),
                            )
                        },
                    )
                    .placeholder("—")
                    .text_size(13)
                    .padding([6, 10]),
                    HoverTarget::SettingsPreviewBaseSize,
                ),
            ]
            .spacing(12)
            .align_y(Alignment::Center),
            iced::widget::Space::new().height(12),
            row![
                text("Outer vertical padding").size(13).color(c.status_text),
                iced::widget::Space::new().width(Fill),
                hover_track(
                    pick_list(
                        px_u16_options(PREVIEW_PADS),
                        Some(PxU16(d.preview.padding_vertical)),
                        |PxU16(p)| {
                            Message::SettingsPreferenceChanged(
                                SettingsPreferenceChanged::PreviewPaddingVertical(p),
                            )
                        },
                    )
                    .placeholder("—")
                    .text_size(13)
                    .padding([6, 10]),
                    HoverTarget::SettingsPreviewPaddingV,
                ),
            ]
            .spacing(12)
            .align_y(Alignment::Center),
            iced::widget::Space::new().height(12),
            row![
                text("Outer horizontal padding").size(13).color(c.status_text),
                iced::widget::Space::new().width(Fill),
                hover_track(
                    pick_list(
                        px_u16_options(PREVIEW_PADS),
                        Some(PxU16(d.preview.padding_horizontal)),
                        |PxU16(p)| {
                            Message::SettingsPreferenceChanged(
                                SettingsPreferenceChanged::PreviewPaddingHorizontal(p),
                            )
                        },
                    )
                    .placeholder("—")
                    .text_size(13)
                    .padding([6, 10]),
                    HoverTarget::SettingsPreviewPaddingH,
                ),
            ]
            .spacing(12)
            .align_y(Alignment::Center),
        ]
        .spacing(0)
        .into(),
        SettingsCategory::Editor => column![
            text("Editor").size(22).color(c.content_text),
            iced::widget::Space::new().height(16),
            row![
                text("Editor font size").size(13).color(c.status_text),
                iced::widget::Space::new().width(Fill),
                hover_track(
                    pick_list(
                        px_u16_options(EDITOR_FONTS),
                        Some(PxU16(d.editor.font_size)),
                        |PxU16(s)| {
                            Message::SettingsPreferenceChanged(
                                SettingsPreferenceChanged::EditorFontSize(s),
                            )
                        },
                    )
                    .placeholder("—")
                    .text_size(13)
                    .padding([6, 10]),
                    HoverTarget::SettingsEditorFontSize,
                ),
            ]
            .spacing(12)
            .align_y(Alignment::Center),
            iced::widget::Space::new().height(12),
            row![
                text("Inner horizontal padding").size(13).color(c.status_text),
                iced::widget::Space::new().width(Fill),
                hover_track(
                    pick_list(
                        px_u16_options(EDITOR_PAD_H),
                        Some(PxU16(d.editor.padding_horizontal)),
                        |PxU16(p)| {
                            Message::SettingsPreferenceChanged(
                                SettingsPreferenceChanged::EditorPaddingHorizontal(p),
                            )
                        },
                    )
                    .placeholder("—")
                    .text_size(13)
                    .padding([6, 10]),
                    HoverTarget::SettingsEditorPaddingH,
                ),
            ]
            .spacing(12)
            .align_y(Alignment::Center),
            iced::widget::Space::new().height(12),
            row![
                text("Inner vertical padding").size(13).color(c.status_text),
                iced::widget::Space::new().width(Fill),
                hover_track(
                    pick_list(
                        px_u16_options(EDITOR_PAD_V),
                        Some(PxU16(d.editor.padding_vertical)),
                        |PxU16(p)| {
                            Message::SettingsPreferenceChanged(
                                SettingsPreferenceChanged::EditorPaddingVertical(p),
                            )
                        },
                    )
                    .placeholder("—")
                    .text_size(13)
                    .padding([6, 10]),
                    HoverTarget::SettingsEditorPaddingV,
                ),
            ]
            .spacing(12)
            .align_y(Alignment::Center),
        ]
        .spacing(0)
        .into(),
        SettingsCategory::Appearance => column![
            text("Appearance").size(22).color(c.content_text),
            iced::widget::Space::new().height(8),
            text("Theme").size(13).color(c.status_text),
            iced::widget::Space::new().height(8),
            hover_track(
                pick_list(
                    vec![ThemeMode::Light, ThemeMode::Dark],
                    Some(app.settings_draft_theme),
                    Message::SettingsDraftThemeSelected,
                )
                .placeholder("Theme")
                .text_size(13)
                .padding([6, 10]),
                HoverTarget::SettingsThemePicker,
            ),
        ]
        .spacing(0)
        .align_x(Alignment::Start)
        .into(),
    };

    let can_save = app.settings_have_unsaved_changes();
    let save_row = row![hover_track(
        button(text("Save").size(13))
            .style(move |theme, status| t::btn_primary(theme, status, mode))
            .padding([8, 18])
            .on_press_maybe(can_save.then(|| Message::SaveSettings)),
        HoverTarget::SettingsSave,
    ),]
    .spacing(8);

    container(
        column![
            panel,
            iced::widget::Space::new().height(Fill),
            save_row,
        ]
        .spacing(0)
        .height(Fill),
    )
    .padding([40, 48])
    .width(Fill)
    .height(Fill)
    .into()
}
