#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod app;
mod constants;
mod fs_utils;
mod ui;

use crate::app::FileManager;
use iced::font::{Family, Stretch, Style, Weight}; // Import necessary font traits
use iced::{Application, Font, Pixels, Settings};
use std::borrow::Cow; // Ensure gstreamer crate is imported

fn main() -> iced::Result {
    let mut settings = Settings::default();

    // Load the custom font data
    let font_data = include_bytes!("../fonts/InterKhmerLooped[wght].ttf");
    settings.fonts = vec![Cow::<'static, [u8]>::Borrowed(font_data)];

    // Set the default font characteristics
    // We assume the loaded font will be used as the default since it's the only one.
    // We specify the desired weight and size.
    settings.default_font = Font {
        family: Family::Name("InterKhmerLooped"), // Use the font family name
        weight: Weight::Medium,                   // 500
        stretch: Stretch::Normal,
        style: Style::Normal,
    };
    settings.default_text_size = Pixels(11.0); // Set default text size to 11 pixels

    let result = FileManager::run(settings);

    // Attempt to save the icon cache on exit
    if let Err(e) = fs_utils::save_icon_cache() {
        eprintln!("Error saving icon cache: {}", e);
    }

    result // Return the result from FileManager::run
}
