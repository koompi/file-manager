use crate::app::{FileManager, Message};
use crate::ui::details_panel; // Import module
use crate::ui::file_grid; // Import module
use crate::ui::sidebar; // Import module
use crate::ui::styles::{BackgroundStyle, RuleStyle};
use crate::ui::top_bar;

// Import module
use iced::widget::{column, container, row, Rule}; // Removed Space import
use iced::{theme, Element, Length};

// The main view function, taking the application state as input
pub fn view(state: &FileManager) -> Element<Message> {
    let sidebar = sidebar::build_sidebar(state); // Use module::function
    let top_bar = top_bar::build_top_bar(state); // Use module::function
    let file_grid = file_grid::build_file_grid(state); // Use module::function
    let details_panel_content = details_panel::details_panel(state); // Corrected function name

    let main_content_area = column![
        top_bar,
        Rule::horizontal(1).style(theme::Rule::Custom(Box::new(RuleStyle))), // Changed Rule::Custom to theme::Rule::Custom
        file_grid
    ]
    .spacing(0);

    // --- Final Layout ---
    // Conditionally create the layout based on the show_details_panel flag
    let main_layout = if state.show_details_panel {
        // Layout WITH details panel (Sidebar | Main (75%) | Details (25%))
        row![
            sidebar,
            Rule::vertical(1).style(theme::Rule::Custom(Box::new(RuleStyle))),
            // Main content takes 3 portions
            container(main_content_area).width(Length::FillPortion(3)),
            Rule::vertical(1).style(theme::Rule::Custom(Box::new(RuleStyle))),
            // Details panel takes 1 portion (25%)
            container(details_panel_content).width(Length::FillPortion(1))
        ]
        .height(Length::Fill)
        .width(Length::Fill)
        .spacing(0)
    } else {
        // Layout WITHOUT details panel (Sidebar | Main (100%))
        row![
            sidebar,
            Rule::vertical(1).style(theme::Rule::Custom(Box::new(RuleStyle))),
            // Main content takes full remaining width
            container(main_content_area).width(Length::Fill)
        ]
        .height(Length::Fill)
        .width(Length::Fill)
        .spacing(0)
    };

    container(main_layout)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(theme::Container::Custom(Box::new(BackgroundStyle))) // Added theme:: prefix
        .into()
}
