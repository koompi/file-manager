use crate::app::{FileManager, GroupCriteria, Message};
use crate::constants::*;
use crate::constants::{FILE_ICON_PATH, FOLDER_ICON_PATH, THUMBNAIL_SIZE};
use crate::fs_utils::DirEntry;
use crate::ui::styles::{SelectedItemStyle, SECONDARY_TEXT_COLOR};
use iced::alignment::{Horizontal, Vertical};
use iced::widget::{button, column, container, image, row, scrollable, text, Column, Rule};
use iced::{theme, Alignment, ContentFit, Element, Length, Renderer, Theme}; // Import ContentFit directly
use iced_aw::Wrap;
use std::collections::BTreeMap;
use std::path::PathBuf; // Import THUMBNAIL_SIZE

const PADDING: f32 = 8.0;
const SPACING: f32 = 10.0;
const ITEM_WIDTH: f32 = 100.0;
const ICON_SIZE: f32 = 52.0;
const MAX_FILENAME_LEN: usize = 15;
const ELLIPSIS: &str = "...";

// Helper function to create a single item widget
fn create_item_widget<'a>(
    entry: &'a DirEntry,
    selected_path: &'a Option<PathBuf>,
) -> Element<'a, Message, Theme, Renderer> {
    let path = entry.path.clone();
    let is_selected = selected_path.as_ref() == Some(&path);

    // Use entry.display_name directly
    let display_name_full = &entry.display_name;

    let display_name = if display_name_full.chars().count() > MAX_FILENAME_LEN {
        format!(
            "{}{}",
            display_name_full
                .chars()
                .take(MAX_FILENAME_LEN - ELLIPSIS.chars().count())
                .collect::<String>(),
            ELLIPSIS
        )
    } else {
        display_name_full.clone() // Clone if not truncated
    };

    // Determine content: Thumbnail, Icon, or Placeholder
    let item_content = if let Some(thumbnail_handle) = &entry.thumbnail {
        // Use thumbnail if available
        image(thumbnail_handle.clone())
            .width(Length::Fixed(THUMBNAIL_SIZE as f32))
            .height(Length::Fixed(THUMBNAIL_SIZE as f32))
            .content_fit(ContentFit::Contain) // Use imported ContentFit
    } else {
        // Use icon if no thumbnail
        let icon_path_string = if entry.is_dir {
            FOLDER_ICON_PATH.to_string() // Convert to String
        } else {
            // Use resolved icon for apps, otherwise generic file icon
            entry
                .resolved_icon_path
                .as_ref()
                .map(|p| p.to_string_lossy().into_owned())
                .unwrap_or_else(|| FILE_ICON_PATH.to_string())
        };
        image(icon_path_string) // Pass String to image()
            .width(Length::Fixed(48.0)) // Keep icon size consistent
            .height(Length::Fixed(48.0))
            .content_fit(ContentFit::Contain) // Use imported ContentFit
    };

    let item_button = button(
        column![
            item_content, // Use the determined content (thumbnail or icon)
            text(display_name) // Use the potentially truncated display_name
                .size(14)
                .horizontal_alignment(Horizontal::Center)
        ]
        .spacing(5)
        .align_items(Alignment::Center)
        .width(Length::Fixed(ITEM_WIDTH)), // Fixed width for grid items
    )
    .style(theme::Button::Text)
    .on_press(Message::ItemClicked(path.clone()));

    let item_container = container(item_button)
        .width(Length::Fixed(ITEM_WIDTH + PADDING))
        .height(Length::Shrink)
        .padding(PADDING / 2.0)
        .center_x()
        .center_y()
        .style(if is_selected {
            theme::Container::Custom(Box::new(SelectedItemStyle))
        } else {
            theme::Container::Transparent
        });

    item_container.into()
}

// Helper function to create a Wrap container for a list of entries
fn create_wrap_for_entries<'a>(
    entries: impl Iterator<Item = &'a DirEntry>,
    selected_path: &'a Option<PathBuf>,
) -> Element<'a, Message, Theme, Renderer> {
    entries
        .fold(Wrap::new(), |wrap_builder, entry| {
            wrap_builder.push(create_item_widget(entry, selected_path))
        })
        .spacing(SPACING)
        .line_spacing(SPACING)
        .into()
}

// Helper function to create a group header
fn create_group_header<'a>(
    group_name: &str,
    item_count: usize,
    is_collapsed: bool,
    group_id: String,
) -> Element<'a, Message> {
    let icon_path = if is_collapsed {
        COLLAPSED_ICON_PATH
    } else {
        EXPANDED_ICON_PATH
    };

    let collapse_button = button(
        image(icon_path)
            .width(Length::Fixed(16.0))
            .height(Length::Fixed(16.0)),
    )
    .on_press(Message::ToggleGroupCollapse(group_id.clone()))
    .style(theme::Button::Text)
    .padding(0);

    let header_text = format!("{} ({})", group_name, item_count);

    row![
        collapse_button,
        text(header_text)
            .style(SECONDARY_TEXT_COLOR)
            .width(Length::Fill)
            .vertical_alignment(Vertical::Center),
    ]
    .spacing(5)
    .align_items(Alignment::Center)
    .width(Length::Fill)
    .into()
}

pub fn build_file_grid(state: &FileManager) -> Element<Message, Theme, Renderer> {
    if let Some(error) = &state.error {
        container(text(error).style(theme::Text::Color(iced::Color::from_rgb8(200, 0, 0))))
            .padding(PADDING * 2.0)
            .center_x()
            .center_y()
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    } else if state.entries.is_empty() {
        container(text("Directory is empty").style(SECONDARY_TEXT_COLOR))
            .padding(PADDING * 2.0)
            .center_x()
            .center_y()
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    } else {
        let content: Element<Message, Theme, Renderer> = match state.group_criteria {
            GroupCriteria::None => {
                let wrap_element =
                    create_wrap_for_entries(state.entries.iter(), &state.selected_path);
                container(wrap_element)
                    .width(Length::Fill)
                    .padding(PADDING)
                    .into()
            }
            GroupCriteria::Type => {
                let (folders, files): (Vec<_>, Vec<_>) =
                    state.entries.iter().partition(|e| e.is_dir);

                let mut main_column = Column::new().spacing(SPACING).padding(PADDING);

                if !folders.is_empty() {
                    let group_id = "folders".to_string();
                    let is_collapsed = state.collapsed_groups.contains(&group_id);
                    main_column = main_column.push(create_group_header(
                        "Folders",
                        folders.len(),
                        is_collapsed,
                        group_id.clone(),
                    ));

                    if !is_collapsed {
                        let folder_element = create_wrap_for_entries(
                            folders.iter().map(|&e| e),
                            &state.selected_path,
                        );
                        main_column = main_column.push(
                            container(folder_element)
                                .width(Length::Fill)
                                .padding([0.0, 0.0, 0.0, 20.0]),
                        );
                    }
                    main_column = main_column.push(Rule::horizontal(1).style(theme::Rule::Default));
                }

                if !files.is_empty() {
                    let group_id = "files".to_string();
                    let is_collapsed = state.collapsed_groups.contains(&group_id);
                    main_column = main_column.push(create_group_header(
                        "Files",
                        files.len(),
                        is_collapsed,
                        group_id.clone(),
                    ));

                    if !is_collapsed {
                        let file_element =
                            create_wrap_for_entries(files.iter().map(|&e| e), &state.selected_path);
                        main_column = main_column.push(
                            container(file_element)
                                .width(Length::Fill)
                                .padding([0.0, 0.0, 0.0, 20.0]),
                        );
                    }
                    main_column = main_column.push(Rule::horizontal(1).style(theme::Rule::Default));
                }

                container(main_column).width(Length::Fill).into()
            }
            GroupCriteria::MimeType => {
                let mut groups: BTreeMap<String, Vec<&DirEntry>> = BTreeMap::new();
                for entry in &state.entries {
                    let group_key = if entry.is_dir {
                        "Folders".to_string()
                    } else {
                        entry
                            .mime_group
                            .clone()
                            .unwrap_or_else(|| "Other".to_string())
                    };
                    groups.entry(group_key).or_default().push(entry);
                }

                let mut main_column = Column::new().spacing(SPACING).padding(PADDING);

                if let Some(folders) = groups.remove("Folders") {
                    let group_id = "folders".to_string();
                    let is_collapsed = state.collapsed_groups.contains(&group_id);
                    main_column = main_column.push(create_group_header(
                        "Folders",
                        folders.len(),
                        is_collapsed,
                        group_id.clone(),
                    ));

                    if !is_collapsed {
                        let folder_element =
                            create_wrap_for_entries(folders.into_iter(), &state.selected_path);
                        main_column = main_column.push(
                            container(folder_element)
                                .width(Length::Fill)
                                .padding([0.0, 0.0, 0.0, 20.0]),
                        );
                    }
                    main_column = main_column.push(Rule::horizontal(1).style(theme::Rule::Default));
                }

                for (group_name, entries) in groups {
                    let group_id = group_name.clone();
                    let is_collapsed = state.collapsed_groups.contains(&group_id);
                    main_column = main_column.push(create_group_header(
                        &group_name,
                        entries.len(),
                        is_collapsed,
                        group_id.clone(),
                    ));

                    if !is_collapsed {
                        let group_element =
                            create_wrap_for_entries(entries.into_iter(), &state.selected_path);
                        main_column = main_column.push(
                            container(group_element)
                                .width(Length::Fill)
                                .padding([0.0, 0.0, 0.0, 20.0]),
                        );
                    }
                    main_column = main_column.push(Rule::horizontal(1).style(theme::Rule::Default));
                }
                container(main_column).width(Length::Fill).into()
            }
        };

        scrollable(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}
