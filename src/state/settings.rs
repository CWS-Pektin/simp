#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SettingsCategory {
    General,
    Preview,
    Editor,
    Appearance,
}

impl SettingsCategory {
    pub const ALL: [SettingsCategory; 4] = [
        SettingsCategory::General,
        SettingsCategory::Preview,
        SettingsCategory::Editor,
        SettingsCategory::Appearance,
    ];

    pub fn label(self) -> &'static str {
        match self {
            SettingsCategory::General => "General",
            SettingsCategory::Preview => "Preview",
            SettingsCategory::Editor => "Editor",
            SettingsCategory::Appearance => "Appearance",
        }
    }
}

/// Where to go after resolving unsaved settings (Save/Discard).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsNavTarget {
    Category(SettingsCategory),
    ExitSettings,
}
