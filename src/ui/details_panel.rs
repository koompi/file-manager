use crate::app::{FileManager, Message};
use crate::constants::{THUMBNAIL_CACHE_DIR, THUMBNAIL_SIZE};
use crate::fs_utils::{self};
use iced::widget::{
    button, column, container, image, scrollable, text,
};
use iced::{Element, Font, Length, Theme, ContentFit, Renderer};
use std::path::PathBuf;
use tokio::task;

// Use Inter font if available, otherwise default
const INTER_REGULAR: Font = Font::with_name("Inter");

// Function to create an action button (with border)
fn action_button<'a>(label: &str, msg: Option<Message>) -> Element<'a, Message> {
    let btn = button(text(label).font(INTER_REGULAR).size(14))
        .style(iced::theme::Button::Secondary)
        .padding(5);

    if let Some(m) = msg {
        btn.on_press(m).into()
    } else {
        btn.into()
    }
}

// Function to load thumbnail asynchronously
async fn load_thumbnail_async(path: PathBuf) -> Option<image::Handle> {
    task::spawn_blocking(move || fs_utils::generate_thumbnail(&path).ok())
        .await
        .ok()
        .flatten()
}

pub fn details_panel(state: &FileManager) -> Element<'_, Message, Theme, Renderer> {
    let content = if let Some(path) = &state.selected_path {
        if let Some(entry) = state.entries.iter().find(|e| e.path == *path) {
            let mut details_column = column![
                text(&entry.display_name).size(20),
                text(format!("Path: {}", entry.path.display())),
            ]
            .spacing(5);

            if entry.is_dir {
                details_column = details_column.push(text("Type: Folder"));
            } else {
                details_column = details_column.push(text(format!(
                    "Type: {}",
                    entry.mime_group.as_deref().unwrap_or("File")
                )));
                details_column =
                    details_column.push(text(format!("Size: {}", fs_utils::format_size(entry.size))));
            }

            if let Some(modified) = entry.modified {
                details_column = details_column
                    .push(text(format!("Modified: {}", fs_utils::format_modified(Some(modified)))));
            }

            // --- Thumbnail Display ---
            if entry.mime_group.as_deref() == Some("Images") {
                if let Some(handle) = &entry.thumbnail {
                    details_column = details_column.push(
                        image(handle.clone())
                            .width(Length::Fixed(128.0))
                            .height(Length::Fixed(128.0))
                            .content_fit(ContentFit::Contain),
                    );
                } else {
                    details_column = details_column.push(text("Loading thumbnail..."));
                }
            }

            container(scrollable(details_column)).padding(10)
        } else {
            container(text("No item selected or item not found."))
                .padding(10)
                .center_x()
                .center_y()
        }
    } else {
        container(text("Select an item to see details."))
            .padding(10)
            .center_x()
            .center_y()
    };

    content.width(Length::Fill).height(Length::Fill).into()
}
