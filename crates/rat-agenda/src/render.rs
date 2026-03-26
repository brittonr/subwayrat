//! Rendering the agenda widget as a ratatui StatefulWidget.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, StatefulWidget, Widget};

use crate::state::{AgendaState, ViewMode};
use crate::types::{AgendaItem, Date};

/// Style for the agenda widget.
#[derive(Debug, Clone)]
pub struct AgendaStyle {
    pub day_header: Style,
    pub time_slot: Style,
    pub priority_a: Style,
    pub priority_b: Style,
    pub priority_c: Style,
    pub todo_style: Style,
    pub done_style: Style,
    pub tags: Style,
    pub today: Style,
    pub selected: Style,
    pub dimmed: Style,
    pub body: Style,
}

impl Default for AgendaStyle {
    fn default() -> Self {
        Self {
            day_header: Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            time_slot: Style::default().fg(Color::DarkGray),
            priority_a: Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            priority_b: Style::default().fg(Color::Yellow),
            priority_c: Style::default().fg(Color::Green),
            todo_style: Style::default().fg(Color::Red),
            done_style: Style::default().fg(Color::Green),
            tags: Style::default().fg(Color::DarkGray),
            today: Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            selected: Style::default().bg(Color::Rgb(40, 40, 60)),
            dimmed: Style::default().fg(Color::DarkGray),
            body: Style::default(),
        }
    }
}

/// The agenda widget.
pub struct Agenda<'a> {
    style: AgendaStyle,
    block: Option<Block<'a>>,
}

impl<'a> Agenda<'a> {
    pub fn new(style: AgendaStyle) -> Self {
        Self { style, block: None }
    }

    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }
}

impl StatefulWidget for Agenda<'_> {
    type State = AgendaState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let inner = if let Some(block) = &self.block {
            let inner = block.inner(area);
            block.clone().render(area, buf);
            inner
        } else {
            area
        };

        if inner.width == 0 || inner.height == 0 {
            return;
        }

        match state.view_mode {
            ViewMode::Day => render_day_view(inner, buf, state, &self.style),
            ViewMode::Week => render_week_view(inner, buf, state, &self.style),
            ViewMode::Month => render_month_view(inner, buf, state, &self.style),
        }
    }
}

fn render_day_view(area: Rect, buf: &mut Buffer, state: &AgendaState, style: &AgendaStyle) {
    let date = state.selected_date;
    let is_today = date == state.today;

    // Header
    let header_style = if is_today { style.today } else { style.day_header };
    let header = format!("{} {} {} {}", date.weekday_name(), date.day, date.month_name(), date.year);
    let header_line = Line::from(Span::styled(header, header_style));
    buf.set_line(area.x, area.y, &header_line, area.width);

    // Items
    let items = state.items_for_date(date);
    let start_row = area.y + 1;
    for (i, item) in items.iter().enumerate() {
        let y = start_row + i as u16;
        if y >= area.y + area.height {
            break;
        }
        let is_selected = i == state.selected_item;
        let line = render_item_line(item, style, area.width as usize);

        if is_selected {
            for x in area.x..area.x + area.width {
                buf[(x, y)].set_style(style.selected);
            }
        }
        buf.set_line(area.x, y, &line, area.width);
    }

    if items.is_empty() {
        let y = start_row;
        if y < area.y + area.height {
            let line = Line::from(Span::styled("  (no items)", style.dimmed));
            buf.set_line(area.x, y, &line, area.width);
        }
    }
}

fn render_week_view(area: Rect, buf: &mut Buffer, state: &AgendaState, style: &AgendaStyle) {
    let week_start = state.selected_date.week_start();
    let col_width = area.width / 7;
    if col_width < 4 {
        return;
    }

    for day_offset in 0..7u32 {
        let date = week_start.add_days(day_offset as i32);
        let x = area.x + (day_offset as u16) * col_width;
        let col_area = Rect::new(x, area.y, col_width, area.height);
        let is_today = date == state.today;
        let is_selected = date == state.selected_date;

        // Day header
        let header_style = if is_today {
            style.today
        } else if is_selected {
            style.day_header.add_modifier(Modifier::UNDERLINED)
        } else {
            style.day_header
        };
        let header = format!("{} {}", date.weekday_name(), date.day);
        buf.set_line(col_area.x, col_area.y, &Line::from(Span::styled(header, header_style)), col_width);

        // Items for this day
        let items = state.items_for_date(date);
        for (i, item) in items.iter().enumerate() {
            let y = col_area.y + 1 + i as u16;
            if y >= col_area.y + col_area.height {
                break;
            }
            // Compact: just show time + truncated title
            let time_str = item.time_start.map(|t| t.to_string()).unwrap_or_default();
            let title_max = col_width as usize - time_str.len() - 1;
            let title: String = item.title.chars().take(title_max).collect();
            let line = Line::from(vec![
                Span::styled(time_str, style.time_slot),
                Span::raw(" "),
                Span::styled(title, style.body),
            ]);
            buf.set_line(col_area.x, y, &line, col_width);
        }
    }
}

fn render_month_view(area: Rect, buf: &mut Buffer, state: &AgendaState, style: &AgendaStyle) {
    let first = state.selected_date.first_of_month();
    let dim = first.days_in_month();
    let first_weekday = first.weekday(); // 0=Mon
    let col_width = area.width / 7;
    if col_width < 3 {
        return;
    }

    // Month/year header
    let header = format!("{} {}", first.month_name(), first.year);
    buf.set_line(area.x, area.y, &Line::from(Span::styled(header, style.day_header)), area.width);

    // Day name headers
    let day_names = ["Mo", "Tu", "We", "Th", "Fr", "Sa", "Su"];
    for (i, name) in day_names.iter().enumerate() {
        let x = area.x + (i as u16) * col_width;
        buf.set_line(x, area.y + 1, &Line::from(Span::styled(*name, style.day_header)), col_width);
    }

    // Day cells
    for day in 1..=dim {
        let date = Date::new(first.year, first.month, day);
        let wd = date.weekday();
        let week_row = (first_weekday + day - 1) / 7;
        let x = area.x + (wd as u16) * col_width;
        let y = area.y + 2 + week_row as u16;
        if y >= area.y + area.height {
            break;
        }

        let is_today = date == state.today;
        let is_selected = date == state.selected_date;
        let item_count = state.items_for_date(date).len();

        let cell_style = if is_today {
            style.today
        } else if is_selected {
            style.selected
        } else if wd >= 5 {
            style.dimmed
        } else {
            style.body
        };

        let label = if item_count > 0 {
            format!("{:2}({}) ", day, item_count)
        } else {
            format!("{:2}    ", day)
        };
        buf.set_line(x, y, &Line::from(Span::styled(label, cell_style)), col_width);
    }
}

fn render_item_line(item: &AgendaItem, style: &AgendaStyle, _width: usize) -> Line<'static> {
    let mut spans: Vec<Span<'static>> = Vec::new();

    // Time
    if let Some(t) = &item.time_start {
        spans.push(Span::styled(format!("{} ", t.to_string()), style.time_slot));
    } else {
        spans.push(Span::styled("      ", style.time_slot));
    }

    // Priority
    if let Some(p) = item.priority {
        let ps = match p {
            'A' => style.priority_a,
            'B' => style.priority_b,
            _ => style.priority_c,
        };
        spans.push(Span::styled(format!("[#{}] ", p), ps));
    }

    // Status
    if let Some(ref s) = item.status {
        let ss = if s == "DONE" || s == "CANCELLED" {
            style.done_style
        } else {
            style.todo_style
        };
        spans.push(Span::styled(format!("{} ", s), ss));
    }

    // Title
    spans.push(Span::styled(item.title.clone(), style.body));

    // Tags
    if !item.tags.is_empty() {
        let tag_str = format!(" :{}:", item.tags.join(":"));
        spans.push(Span::styled(tag_str, style.tags));
    }

    Line::from(spans)
}
