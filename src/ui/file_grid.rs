use crate::app::{FileManager, GroupCriteria, Message};
use crate::constants::*;
use crate::fs_utils::DirEntry;
use crate::ui::styles::{SelectedItemStyle, SECONDARY_TEXT_COLOR};
use iced::alignment::{Horizontal, Vertical};
use iced::widget::{button, column, container, image, row, scrollable, text, Column, Rule};
use iced::{theme, Alignment, Element, Length, Renderer, Theme};
use iced_aw::Wrap;
use std::collections::BTreeMap;
use std::path::PathBuf;

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

    let display_name_full = path
        .file_name()
        .map_or_else(|| "..".into(), |name| name.to_string_lossy().into_owned());

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
        display_name_full
    };

    let icon_path = if entry.is_dir {
        FOLDER_ICON_PATH
    } else {
        FILE_ICON_PATH
    };

    let entry_icon = image(icon_path)
        .height(Length::Fixed(ICON_SIZE))
        .width(Length::Fixed(ICON_SIZE));

    let item_content = column![
        entry_icon,
        text(display_name)
            .width(Length::Fixed(ITEM_WIDTH))
            .horizontal_alignment(Horizontal::Center)
    ]
    .spacing(5)
    .align_items(Alignment::Center)
    .width(Length::Fixed(ITEM_WIDTH));

    let item_button = button(item_content)
        .on_press(Message::ItemClicked(path.clone()))
        .style(theme::Button::Text)
        .padding(0);

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
