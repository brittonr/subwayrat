//! Styles for date picker widgets.

use ratatui::style::{Color, Modifier, Style};

#[derive(Debug, Clone)]
pub struct CalendarStyle {
    pub selected: Style,
    pub today: Style,
    pub weekend: Style,
    pub weekday_header: Style,
    pub out_of_month: Style,
    pub title: Style,
    pub body: Style,
}

impl Default for CalendarStyle {
    fn default() -> Self {
        Self {
            selected: Style::default()
                .bg(Color::Rgb(50, 50, 80))
                .add_modifier(Modifier::BOLD),
            today: Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
            weekend: Style::default().fg(Color::DarkGray),
            weekday_header: Style::default().fg(Color::Cyan),
            out_of_month: Style::default().fg(Color::Rgb(60, 60, 60)),
            title: Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
            body: Style::default(),
        }
    }
}
