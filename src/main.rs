#![cfg_attr(windows, windows_subsystem = "windows")]

mod app;
mod fs;
mod message;
mod persist;
mod state;
mod ui;
mod window_icon;

pub fn main() -> iced::Result {
    app::run()
}
