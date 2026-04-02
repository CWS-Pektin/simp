#[derive(Debug, Clone)]
pub enum StatusState {
    Idle,
    Saving,
    Saved,
    Error(String),
    Conflict(String),
}

impl Default for StatusState {
    fn default() -> Self {
        Self::Idle
    }
}
