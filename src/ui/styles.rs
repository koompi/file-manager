use iced::widget::rule;
use iced::{Background, Border, Color, Theme, Vector};

// Define some theme colors (adapt these to your preference)
pub const BACKGROUND_COLOR: Color = Color::from_rgb(0.95, 0.95, 0.95); // Light gray background
pub const ACCENT_COLOR: Color = Color::from_rgb(0.3, 0.55, 0.75); // Less saturated blue accent
pub const SELECTED_BG_COLOR: Color = Color::from_rgba(0.3, 0.55, 0.75, 0.15); // Lighter, less saturated accent for selection
pub const BORDER_COLOR: Color = Color::from_rgb(0.75, 0.75, 0.75); // Made slightly darker
pub const TEXT_COLOR: Color = Color::from_rgb(0.2, 0.2, 0.2);
pub const SECONDARY_TEXT_COLOR: Color = Color::from_rgb(0.5, 0.5, 0.5);

// Custom Button Style for Sidebar/Breadcrumbs (subtle)
pub struct LinkButtonStyle;
impl iced::widget::button::StyleSheet for LinkButtonStyle {
    type Style = Theme;
    fn active(&self, _style: &Self::Style) -> iced::widget::button::Appearance {
        iced::widget::button::Appearance {
            text_color: ACCENT_COLOR, // Use accent color for text
            background: None,
            border: Border::default(),
            shadow: iced::Shadow::default(),
            shadow_offset: Vector::default(),
        }
    }
    fn hovered(&self, style: &Self::Style) -> iced::widget::button::Appearance {
        let active = self.active(style);
        iced::widget::button::Appearance {
            text_color: Color {
                a: 0.8,
                ..active.text_color
            }, // Slightly fade on hover
            ..active
        }
    }
}

// Custom Container Style for selected items
pub struct SelectedItemStyle;
impl iced::widget::container::StyleSheet for SelectedItemStyle {
    type Style = Theme;
    fn appearance(&self, _style: &Self::Style) -> iced::widget::container::Appearance {
        iced::widget::container::Appearance {
            text_color: Some(TEXT_COLOR),
            background: Some(Background::Color(SELECTED_BG_COLOR)),
            border: Border {
                radius: 4.0.into(),
                width: 1.0,          // Add a subtle border
                color: ACCENT_COLOR, // Use accent color for border
            },
            shadow: iced::Shadow::default(),
        }
    }
}

// Custom Container Style for the main background
pub struct BackgroundStyle;
impl iced::widget::container::StyleSheet for BackgroundStyle {
    type Style = Theme;
    fn appearance(&self, _style: &Self::Style) -> iced::widget::container::Appearance {
        iced::widget::container::Appearance {
            text_color: Some(TEXT_COLOR),
            background: Some(Background::Color(BACKGROUND_COLOR)),
            border: Border::default(),
            shadow: iced::Shadow::default(),
        }
    }
}

// Original style for single segment or fallback
pub struct BreadcrumbSegmentStyle;
impl iced::widget::container::StyleSheet for BreadcrumbSegmentStyle {
    type Style = Theme;
    fn appearance(&self, _style: &Self::Style) -> iced::widget::container::Appearance {
        iced::widget::container::Appearance {
            text_color: None, // Inherit from button
            background: None, // Transparent background
            border: Border {
                radius: 3.0.into(), // Original radius
                width: 1.0,
                color: BORDER_COLOR,
            },
            shadow: iced::Shadow::default(),
        }
    }
}

// Style for the first breadcrumb segment
pub struct BreadcrumbStartSegmentStyle;
impl iced::widget::container::StyleSheet for BreadcrumbStartSegmentStyle {
    type Style = Theme;
    fn appearance(&self, _style: &Self::Style) -> iced::widget::container::Appearance {
        iced::widget::container::Appearance {
            text_color: None,
            background: None,
            border: Border {
                radius: iced::border::Radius::from([3.0, 0.0, 0.0, 3.0]), // Radius top-left, bottom-left
                width: 1.0,
                color: BORDER_COLOR,
            },
            shadow: iced::Shadow::default(),
        }
    }
}

// Style for middle breadcrumb segments
pub struct BreadcrumbMiddleSegmentStyle;
impl iced::widget::container::StyleSheet for BreadcrumbMiddleSegmentStyle {
    type Style = Theme;
    fn appearance(&self, _style: &Self::Style) -> iced::widget::container::Appearance {
        iced::widget::container::Appearance {
            text_color: None,
            background: None,
            border: Border {
                radius: 0.0.into(), // No radius
                width: 1.0,
                color: BORDER_COLOR,
            },
            shadow: iced::Shadow::default(),
        }
    }
}

// Style for the last breadcrumb segment
pub struct BreadcrumbEndSegmentStyle;
impl iced::widget::container::StyleSheet for BreadcrumbEndSegmentStyle {
    type Style = Theme;
    fn appearance(&self, _style: &Self::Style) -> iced::widget::container::Appearance {
        iced::widget::container::Appearance {
            text_color: None,
            background: None,
            border: Border {
                radius: iced::border::Radius::from([0.0, 3.0, 3.0, 0.0]), // Radius top-right, bottom-right
                width: 1.0,
                color: BORDER_COLOR,
            },
            shadow: iced::Shadow::default(),
        }
    }
}

// Style for the Back navigation button (radius left)
pub struct NavBackButtonStartStyle;
impl iced::widget::container::StyleSheet for NavBackButtonStartStyle {
    type Style = Theme;
    fn appearance(&self, _style: &Self::Style) -> iced::widget::container::Appearance {
        iced::widget::container::Appearance {
            text_color: None,
            background: None,
            border: Border {
                radius: iced::border::Radius::from([3.0, 0.0, 0.0, 3.0]), // Radius top-left, bottom-left
                width: 1.0,
                color: BORDER_COLOR,
            },
            shadow: iced::Shadow::default(),
        }
    }
}

// Style for the Forward navigation button (no radius)
pub struct NavButtonMiddleStyle;
impl iced::widget::container::StyleSheet for NavButtonMiddleStyle {
    type Style = Theme;
    fn appearance(&self, _style: &Self::Style) -> iced::widget::container::Appearance {
        iced::widget::container::Appearance {
            text_color: None,
            background: None,
            border: Border {
                radius: 0.0.into(), // No radius
                width: 1.0,
                color: BORDER_COLOR,
            },
            shadow: iced::Shadow::default(),
        }
    }
}

// Style for the Up navigation button (radius right)
pub struct NavButtonEndStyle;
impl iced::widget::container::StyleSheet for NavButtonEndStyle {
    type Style = Theme;
    fn appearance(&self, _style: &Self::Style) -> iced::widget::container::Appearance {
        iced::widget::container::Appearance {
            text_color: None,
            background: None,
            border: Border {
                radius: iced::border::Radius::from([0.0, 3.0, 3.0, 0.0]), // Radius top-right, bottom-right
                width: 1.0,
                color: BORDER_COLOR,
            },
            shadow: iced::Shadow::default(),
        }
    }
}

// Custom Rule Style
pub struct RuleStyle;
impl iced::widget::rule::StyleSheet for RuleStyle {
    type Style = Theme;
    fn appearance(&self, _style: &Self::Style) -> iced::widget::rule::Appearance {
        iced::widget::rule::Appearance {
            color: BORDER_COLOR, // Use border color
            width: 1,
            radius: 0.0.into(),
            fill_mode: rule::FillMode::Full,
        }
    }
}
