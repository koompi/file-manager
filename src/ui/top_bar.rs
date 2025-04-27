use crate::app::{FileManager, GroupCriteria, Message, SortCriteria, SortOrder};
use crate::constants::*;
use crate::ui::styles::{
    BreadcrumbEndSegmentStyle, BreadcrumbMiddleSegmentStyle, BreadcrumbSegmentStyle,
    BreadcrumbStartSegmentStyle, LinkButtonStyle, NavBackButtonStartStyle, NavButtonEndStyle,
    NavButtonMiddleStyle,
};
use iced::widget::{button, checkbox, container, image, row, text, Space};
use iced::{theme, Alignment, Element, Length, Theme};
use std::path::{Component, PathBuf};

const PADDING: f32 = 8.0;
const SPACING: f32 = 10.0;
const BUTTON_HEIGHT: f32 = 28.0; // Define a consistent height
const NAV_ICON_SIZE: f32 = 16.0; // Size for nav icons
const SORT_ICON_SIZE: f32 = 16.0; // Size for sort icons
const SORT_BUTTON_PADDING: f32 = 6.0; // Padding for sort icon buttons
const BREADCRUMB_TEXT_SIZE: u16 = 14; // Keep text size for breadcrumbs
const TOGGLE_PANEL_ICON_SIZE: f32 = 16.0; // Size for the new toggle icon

pub fn build_top_bar(state: &FileManager) -> Element<Message> {
    // --- Navigation Buttons ---
    let back_button_inner = button(
        image(BACK_ICON_PATH)
            .width(Length::Fixed(NAV_ICON_SIZE))
            .height(Length::Fixed(NAV_ICON_SIZE)),
    )
    .on_press_maybe(state.can_go_back().then_some(Message::GoBack))
    .style(theme::Button::Secondary);
    let back_button = container(back_button_inner)
        .width(Length::Fixed(BUTTON_HEIGHT))
        .height(Length::Fixed(BUTTON_HEIGHT))
        .center_x()
        .center_y()
        .style(theme::Container::Custom(Box::new(NavBackButtonStartStyle)));

    let forward_button_inner = button(
        image(FORWARD_ICON_PATH)
            .width(Length::Fixed(NAV_ICON_SIZE))
            .height(Length::Fixed(NAV_ICON_SIZE)),
    )
    .on_press_maybe(state.can_go_forward().then_some(Message::GoForward))
    .style(theme::Button::Secondary);
    let forward_button = container(forward_button_inner)
        .width(Length::Fixed(BUTTON_HEIGHT))
        .height(Length::Fixed(BUTTON_HEIGHT))
        .center_x()
        .center_y()
        .style(theme::Container::Custom(Box::new(NavButtonMiddleStyle)));

    let up_button_inner = button(
        image(UP_ICON_PATH)
            .width(Length::Fixed(NAV_ICON_SIZE))
            .height(Length::Fixed(NAV_ICON_SIZE)),
    )
    .on_press(Message::GoUp)
    .style(theme::Button::Secondary);
    let up_button = container(up_button_inner)
        .width(Length::Fixed(BUTTON_HEIGHT))
        .height(Length::Fixed(BUTTON_HEIGHT))
        .center_x()
        .center_y()
        .style(theme::Container::Custom(Box::new(NavButtonEndStyle)));

    let navigation_buttons = row![back_button, forward_button, up_button]
        .spacing(-1.0) // Negative spacing to make borders overlap
        .align_items(Alignment::Center);

    // --- Breadcrumbs ---
    let mut breadcrumbs = row![].align_items(Alignment::Center).spacing(-1.0); // Negative spacing
    let mut current_breadcrumb_path = PathBuf::new();

    let normal_components: Vec<_> = state
        .current_path
        .components()
        .filter_map(|c| {
            if let Component::Normal(name) = c {
                Some(name)
            } else {
                None
            }
        })
        .collect();
    let has_root = state.current_path.has_root();
    let total_segments = if has_root { 1 } else { 0 } + normal_components.len();

    let mut current_segment_index = 0;

    if has_root {
        let root_path = PathBuf::from("/");
        let root_button = button(text("Root"))
            .on_press(Message::Navigate(root_path))
            .style(theme::Button::Custom(Box::new(LinkButtonStyle)))
            .padding([PADDING / 2.0, PADDING, PADDING / 2.0, PADDING]);

        let style: Box<dyn container::StyleSheet<Style = Theme>> = if total_segments == 1 {
            Box::new(BreadcrumbSegmentStyle)
        } else {
            Box::new(BreadcrumbStartSegmentStyle)
        };

        breadcrumbs = breadcrumbs.push(
            container(root_button)
                .width(Length::Shrink)
                .height(Length::Fixed(BUTTON_HEIGHT))
                .center_y()
                .style(theme::Container::Custom(style)),
        );
        current_breadcrumb_path.push("/");
        current_segment_index += 1;
    }

    for name in normal_components.iter() {
        let name_str = name.to_string_lossy();
        current_breadcrumb_path.push(name);
        let path_for_button = current_breadcrumb_path.clone();

        let segment_button = button(text(name_str))
            .on_press(Message::Navigate(path_for_button))
            .style(theme::Button::Custom(Box::new(LinkButtonStyle)))
            .padding([PADDING / 2.0, PADDING, PADDING / 2.0, PADDING]);

        let style: Box<dyn container::StyleSheet<Style = Theme>> = if total_segments == 1 {
            Box::new(BreadcrumbSegmentStyle)
        } else if current_segment_index == 0 && has_root {
            // If it's the first *after* root
            Box::new(BreadcrumbMiddleSegmentStyle)
        } else if current_segment_index == 0 && !has_root {
            // If it's the very first segment (no root)
            Box::new(BreadcrumbStartSegmentStyle)
        } else if current_segment_index == total_segments - 1 {
            Box::new(BreadcrumbEndSegmentStyle)
        } else {
            Box::new(BreadcrumbMiddleSegmentStyle)
        };

        breadcrumbs = breadcrumbs.push(
            container(segment_button)
                .width(Length::Shrink)
                .height(Length::Fixed(BUTTON_HEIGHT))
                .center_y()
                .style(theme::Container::Custom(style)),
        );
        current_segment_index += 1;
    }

    // --- Toggle Hidden Files Checkbox ---
    let toggle_hidden_checkbox = checkbox(".file", state.show_hidden_files)
        .on_toggle(|_| Message::ToggleHiddenFiles) // Send the toggle message regardless of new state
        .spacing(SPACING / 2.0);

    // --- Sorting Buttons ---
    let sort_name_icon = match state.sort_order {
        SortOrder::Ascending => SORT_NAME_ASC_ICON_PATH,
        SortOrder::Descending => SORT_NAME_DESC_ICON_PATH,
    };
    let sort_name_button_inner = button(
        image(sort_name_icon)
            .width(Length::Fixed(SORT_ICON_SIZE))
            .height(Length::Fixed(SORT_ICON_SIZE)),
    )
    .on_press(Message::SetSortCriteria(SortCriteria::Name))
    .style(if state.sort_criteria == SortCriteria::Name {
        theme::Button::Primary // Highlight active sort
    } else {
        theme::Button::Secondary
    })
    .padding(SORT_BUTTON_PADDING);
    let sort_name_button = container(sort_name_button_inner)
        .width(Length::Fixed(BUTTON_HEIGHT))
        .height(Length::Fixed(BUTTON_HEIGHT))
        .center_x()
        .center_y()
        .style(theme::Container::Custom(Box::new(NavBackButtonStartStyle))); // Start style

    let sort_size_icon = match state.sort_order {
        SortOrder::Ascending => SORT_SIZE_ASC_ICON_PATH,
        SortOrder::Descending => SORT_SIZE_DESC_ICON_PATH,
    };
    let sort_size_button_inner = button(
        image(sort_size_icon)
            .width(Length::Fixed(SORT_ICON_SIZE))
            .height(Length::Fixed(SORT_ICON_SIZE)),
    )
    .on_press(Message::SetSortCriteria(SortCriteria::Size))
    .style(if state.sort_criteria == SortCriteria::Size {
        theme::Button::Primary
    } else {
        theme::Button::Secondary
    })
    .padding(SORT_BUTTON_PADDING);
    let sort_size_button = container(sort_size_button_inner)
        .width(Length::Fixed(BUTTON_HEIGHT))
        .height(Length::Fixed(BUTTON_HEIGHT))
        .center_x()
        .center_y()
        .style(theme::Container::Custom(Box::new(NavButtonMiddleStyle))); // Middle style

    let sort_date_icon = match state.sort_order {
        SortOrder::Ascending => SORT_DATE_ASC_ICON_PATH,
        SortOrder::Descending => SORT_DATE_DESC_ICON_PATH,
    };
    let sort_date_button_inner = button(
        image(sort_date_icon)
            .width(Length::Fixed(SORT_ICON_SIZE))
            .height(Length::Fixed(SORT_ICON_SIZE)),
    )
    .on_press(Message::SetSortCriteria(SortCriteria::ModifiedDate))
    .style(if state.sort_criteria == SortCriteria::ModifiedDate {
        theme::Button::Primary
    } else {
        theme::Button::Secondary
    })
    .padding(SORT_BUTTON_PADDING);
    let sort_date_button = container(sort_date_button_inner)
        .width(Length::Fixed(BUTTON_HEIGHT))
        .height(Length::Fixed(BUTTON_HEIGHT))
        .center_x()
        .center_y()
        .style(theme::Container::Custom(Box::new(NavButtonMiddleStyle))); // Middle style

    let sort_type_icon = match state.sort_order {
        SortOrder::Ascending => SORT_TYPE_ASC_ICON_PATH,
        SortOrder::Descending => SORT_TYPE_DESC_ICON_PATH,
    };
    let sort_type_button_inner = button(
        image(sort_type_icon)
            .width(Length::Fixed(SORT_ICON_SIZE))
            .height(Length::Fixed(SORT_ICON_SIZE)),
    )
    .on_press(Message::SetSortCriteria(SortCriteria::Type))
    .style(if state.sort_criteria == SortCriteria::Type {
        theme::Button::Primary
    } else {
        theme::Button::Secondary
    })
    .padding(SORT_BUTTON_PADDING);
    let sort_type_button = container(sort_type_button_inner)
        .width(Length::Fixed(BUTTON_HEIGHT))
        .height(Length::Fixed(BUTTON_HEIGHT))
        .center_x()
        .center_y()
        .style(theme::Container::Custom(Box::new(NavButtonEndStyle))); // End style

    let sorting_controls = row![
        sort_name_button,
        sort_size_button,
        sort_date_button,
        sort_type_button,
    ]
    .spacing(-1.0) // Use negative spacing for overlap
    .align_items(Alignment::Center);
    // --- End Sorting Buttons ---

    // --- Grouping Controls ---
    let is_grouped_by_category = state.group_criteria == GroupCriteria::MimeType;
    let group_by_category_checkbox = checkbox("Group", is_grouped_by_category)
        .on_toggle(|is_checked| {
            if is_checked {
                Message::SetGroupCriteria(GroupCriteria::MimeType)
            } else {
                Message::SetGroupCriteria(GroupCriteria::None)
            }
        })
        .spacing(SPACING / 2.0);

    let grouping_controls = row![group_by_category_checkbox]
        .spacing(SPACING / 2.0)
        .align_items(Alignment::Center);
    // --- End Grouping Controls ---

    // --- Toggle Details Panel Button ---
    let toggle_panel_icon = if state.show_details_panel {
        FORWARD_ICON_PATH // Placeholder, replace with a better icon
    } else {
        BACK_ICON_PATH // Placeholder, replace with a better icon
    };

    let toggle_panel_button_inner = button(
        image(toggle_panel_icon)
            .width(Length::Fixed(TOGGLE_PANEL_ICON_SIZE))
            .height(Length::Fixed(TOGGLE_PANEL_ICON_SIZE)),
    )
    .on_press(Message::ToggleDetailsPanel)
    .style(theme::Button::Secondary)
    .padding(SORT_BUTTON_PADDING);

    let toggle_panel_button = container(toggle_panel_button_inner)
        .width(Length::Fixed(BUTTON_HEIGHT))
        .height(Length::Fixed(BUTTON_HEIGHT))
        .center_x()
        .center_y()
        .style(theme::Container::Custom(Box::new(NavButtonEndStyle)));
    // --- End Toggle Details Panel Button ---

    row![
        navigation_buttons,
        Space::with_width(Length::Fixed(SPACING / 2.0)),
        breadcrumbs,
        Space::with_width(Length::Fill), // Push controls to the right
        toggle_hidden_checkbox,          // Use the checkbox here
        Space::with_width(Length::Fixed(SPACING / 2.0)), // Add spacing
        grouping_controls,               // Add grouping controls
        Space::with_width(Length::Fixed(SPACING / 2.0)), // Add spacing
        sorting_controls,                // Add sorting controls
        Space::with_width(Length::Fixed(SPACING / 2.0)), // Add spacing
        toggle_panel_button,             // Add the new toggle button
    ]
    .padding(PADDING)
    .spacing(SPACING)
    .align_items(Alignment::Center)
    .width(Length::Fill)
    .into()
}
