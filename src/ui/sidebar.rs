use crate::app::{FileManager, Message};
use crate::constants::*;
use crate::ui::styles::RuleStyle;
use iced::widget::{button, column, container, image, row, text, Rule, Space};
use iced::{theme, Alignment, Element, Length};
use std::path::PathBuf;

const SIDEBAR_ICON_SIZE: f32 = 24.0; // Slightly larger icons
const PADDING: f32 = 8.0;
const SPACING: f32 = 10.0;

// Helper for sidebar buttons
fn sidebar_button_content(icon_path: &str, label: &str) -> Element<'static, Message> {
    row![
        image(icon_path)
            .height(Length::Fixed(SIDEBAR_ICON_SIZE))
            .width(Length::Fixed(SIDEBAR_ICON_SIZE)),
        text(label) // Removed .size(14)
    ]
    .spacing(8)
    .align_items(Alignment::Center)
    .into()
}

pub fn build_sidebar(_state: &FileManager) -> Element<Message> {
    let mut sidebar_content = column![
        Space::with_height(Length::Fixed(PADDING)),
        button(sidebar_button_content(HOME_ICON_PATH, "Home"))
            .on_press(Message::Navigate(
                dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"))
            ))
            .style(theme::Button::Text)
            .width(Length::Fill)
            .padding(PADDING),
        button(sidebar_button_content(ROOT_ICON_PATH, "Root"))
            .on_press(Message::Navigate(PathBuf::from("/")))
            .style(theme::Button::Text)
            .width(Length::Fill)
            .padding(PADDING),
        Rule::horizontal(1).style(theme::Rule::Custom(Box::new(RuleStyle))),
    ]
    .spacing(SPACING / 2.0)
    .padding(PADDING);

    let user_dirs = [
        ("Desktop", DESKTOP_ICON_PATH, dirs::desktop_dir()),
        ("Documents", DOCUMENTS_ICON_PATH, dirs::document_dir()),
        ("Downloads", DOWNLOADS_ICON_PATH, dirs::download_dir()),
        ("Music", MUSIC_ICON_PATH, dirs::audio_dir()),
        ("Pictures", PICTURES_ICON_PATH, dirs::picture_dir()),
        ("Videos", VIDEOS_ICON_PATH, dirs::video_dir()),
    ];

    for (label, icon_path, path_opt) in user_dirs {
        if let Some(path) = path_opt {
            sidebar_content = sidebar_content.push(
                button(sidebar_button_content(icon_path, label))
                    .on_press(Message::Navigate(path))
                    .style(theme::Button::Text)
                    .width(Length::Fill)
                    .padding(PADDING),
            );
        }
    }
    sidebar_content = sidebar_content.push(Space::with_height(Length::Fill));

    container(sidebar_content)
        .width(Length::Fixed(180.0))
        .height(Length::Fill)
        .style(theme::Container::Transparent)
        .into()
}
