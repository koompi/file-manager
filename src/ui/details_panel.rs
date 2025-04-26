use crate::app::{FileManager, Message};
use crate::fs_utils::{format_modified, format_size, PreviewContent};
use iced::widget::{
    button, column, container, image, row, scrollable, text, text_input, Column, Rule, Space,
};
use iced::{Alignment, Element, Font, Length};
use std::fs;

// Use Inter font if available, otherwise default
const INTER_REGULAR: Font = Font::with_name("Inter");

// Function to create an action button (with border)
fn action_button<'a>(label: &str, msg: Option<Message>) -> Element<'a, Message> {
    let btn = button(text(label).font(INTER_REGULAR).size(14))
        // Use Secondary style which typically has a border in default themes
        .style(iced::theme::Button::Secondary)
        .padding(5);

    // Disabled state is handled by the theme for Secondary buttons
    if let Some(m) = msg {
        btn.on_press(m).into()
    } else {
        btn.into()
    }
}

pub fn details_panel(state: &FileManager) -> Element<Message> {
    // Find the selected entry details if a path is selected
    let selected_entry = state.selected_path.as_ref().and_then(|selected_path| {
        state
            .entries
            .iter()
            .find(|entry| entry.path == *selected_path)
    });

    // Determine content based on whether an entry is selected OR show CWD details
    let (name_display, type_str, size_str, modified_str, mime_str, action_buttons): (
        Element<Message>,
        String,
        String,
        String,
        Element<Message>,
        Element<Message>,
    ) = if let Some(entry) = selected_entry {
        // --- Item Selected ---
        let name_widget: Element<Message> = if state.is_renaming(&entry.path) {
            text_input("New name...", &state.rename_input_value)
                .on_input(Message::RenameInputChanged)
                .on_submit(Message::ConfirmRename)
                .padding(5)
                .font(INTER_REGULAR)
                .into()
        } else {
            text(entry.path.file_name().unwrap_or_default().to_string_lossy())
                .font(INTER_REGULAR)
                .size(18)
                .into()
        };

        let type_info = if entry.is_dir { "Folder" } else { "File" }.to_string();
        let size_info = format_size(entry.size);
        let modified_info = format_modified(entry.modified);
        let mime_info: Element<Message> = if !entry.is_dir {
            if let Some(mime_group) = &entry.mime_group {
                Element::from(
                    text(format!("MIME Group: {}", mime_group))
                        .font(INTER_REGULAR)
                        .size(14),
                )
            } else {
                Element::from(text("MIME Group: Unknown").font(INTER_REGULAR).size(14))
            }
        } else {
            Element::from(Space::new(Length::Shrink, Length::Shrink))
        };

        let buttons = row![
            action_button("Copy", Some(Message::CopyItem(entry.path.clone()))),
            action_button("Cut", Some(Message::CutItem(entry.path.clone()))),
            action_button(
                "Paste",
                if state.clipboard_item.is_some() {
                    Some(Message::Paste)
                } else {
                    None
                }
            ),
            action_button("Rename", Some(Message::StartRename(entry.path.clone()))),
            action_button("Delete", Some(Message::DeleteItem(entry.path.clone()))),
        ]
        .spacing(15)
        .align_items(Alignment::Center);

        (
            name_widget,
            type_info,
            size_info,
            modified_info,
            mime_info,
            buttons.into(),
        )
    } else {
        // --- No Item Selected - Show CWD Details ---
        let cwd_path = &state.current_path;
        let (name_widget, type_info, size_info, modified_info, mime_info_elem) =
            match fs::metadata(cwd_path) {
                Ok(metadata) => {
                    let name = cwd_path
                        .file_name()
                        .map(|n| n.to_string_lossy().into_owned())
                        .unwrap_or_else(|| "/".to_string());
                    let name_widget: Element<Message> =
                        text(name).font(INTER_REGULAR).size(18).into();
                    let type_info = "Folder".to_string();
                    let size_info = "-".to_string();
                    let modified_info = format_modified(metadata.modified().ok());
                    let mime_info_elem = Element::from(Space::new(Length::Shrink, Length::Shrink));

                    (
                        name_widget,
                        type_info,
                        size_info,
                        modified_info,
                        mime_info_elem,
                    )
                }
                Err(_) => {
                    let name_placeholder = text("Directory details unavailable")
                        .font(INTER_REGULAR)
                        .size(18)
                        .into();
                    let type_info = "-".to_string();
                    let size_info = "-".to_string();
                    let modified_info = "-".to_string();
                    let mime_info_elem =
                        Element::from(text("MIME Group: -").font(INTER_REGULAR).size(14));
                    (
                        name_placeholder,
                        type_info,
                        size_info,
                        modified_info,
                        mime_info_elem,
                    )
                }
            };

        let buttons = row![
            action_button("Copy", None),
            action_button("Cut", None),
            action_button(
                "Paste",
                if state.clipboard_item.is_some() {
                    Some(Message::Paste)
                } else {
                    None
                }
            ),
            action_button("Rename", None),
            action_button("Delete", None),
        ]
        .spacing(15)
        .align_items(Alignment::Center);

        (
            name_widget,
            type_info,
            size_info,
            modified_info,
            mime_info_elem,
            buttons.into(),
        )
    };

    // --- Preview Area ---
    let preview_area: Element<Message> = if state.selected_path.is_some() {
        match &state.preview_content {
            Some(PreviewContent::Text(content)) => {
                container(scrollable(text(content).font(INTER_REGULAR).size(12)))
                    .height(Length::Fixed(200.0))
                    .width(Length::Fill)
                    .style(iced::theme::Container::Box)
                    .padding(5)
                    .into()
            }
            Some(PreviewContent::Image(handle)) => container(
                image(handle.clone())
                    .width(Length::Fill)
                    .content_fit(iced::ContentFit::Contain),
            )
            .width(Length::Fill)
            .height(Length::Fixed(200.0))
            .center_x()
            .center_y()
            .style(iced::theme::Container::Box)
            .padding(5)
            .into(),
            Some(PreviewContent::Error(e)) => container(
                text(format!("Preview Error: {}", e))
                    .font(INTER_REGULAR)
                    .size(12),
            )
            .height(Length::Fixed(200.0))
            .width(Length::Fill)
            .center_x()
            .center_y()
            .style(iced::theme::Container::Box)
            .padding(5)
            .into(),
            None => {
                let should_preview = state
                    .selected_path
                    .as_ref()
                    .and_then(|p| state.entries.iter().find(|e| e.path == *p))
                    .map_or(false, |entry| {
                        !entry.is_dir
                            && entry.mime_group.as_ref().map_or(false, |group| {
                                group == "Images" || group == "Text Files" || group == "Videos"
                            })
                    });

                if should_preview {
                    container(text("Loading preview...").font(INTER_REGULAR).size(12))
                        .height(Length::Fixed(200.0))
                        .width(Length::Fill)
                        .center_x()
                        .center_y()
                        .style(iced::theme::Container::Box)
                        .padding(5)
                        .into()
                } else {
                    Space::new(Length::Fill, Length::Fixed(200.0)).into()
                }
            }
        }
    } else {
        Space::new(Length::Fill, Length::Fixed(200.0)).into()
    };
    // --- End Preview Area ---

    // --- Metadata Section ---
    let metadata_section = column![
        name_display,
        Space::new(Length::Fill, Length::Fixed(5.0)),
        text(format!("Type: {}", type_str))
            .font(INTER_REGULAR)
            .size(14),
        text(format!("Size: {}", size_str))
            .font(INTER_REGULAR)
            .size(14),
        text(format!("Modified: {}", modified_str))
            .font(INTER_REGULAR)
            .size(14),
        mime_str,
    ]
    .spacing(4);

    // Build the main column structure
    let content_column = Column::new()
        .push(metadata_section)
        .push(Space::new(Length::Fill, Length::Fixed(15.0)))
        .push(Rule::horizontal(1))
        .push(Space::new(Length::Fill, Length::Fixed(15.0)))
        .push(preview_area)
        .push(Space::new(Length::Fill, Length::Fixed(15.0)))
        .push(Rule::horizontal(1))
        .push(Space::new(Length::Fill, Length::Fixed(15.0)))
        .push(action_buttons)
        .spacing(0)
        .padding(15)
        .width(Length::Fill)
        .align_items(Alignment::Start);

    // Wrap in container and scrollable
    container(scrollable(content_column))
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(0)
        .into()
}
