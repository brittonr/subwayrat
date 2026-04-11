//! Backlinks panel: displays incoming references grouped by file.

use std::collections::HashMap;

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, StatefulWidget, Widget};

// ── Types ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct Backlink {
    pub source_file: String,
    pub source_heading: Option<String>,
    pub source_line: usize,
    pub context_before: String,
    pub context_line: String,
    pub context_after: String,
    pub link_text: String,
}

pub trait BacklinkSource {
    fn backlinks(&self, target_id: &str) -> Vec<Backlink>;
}

impl BacklinkSource for Vec<Backlink> {
    fn backlinks(&self, _target_id: &str) -> Vec<Backlink> {
        self.clone()
    }
}

// ── State ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
struct FileGroup {
    file: String,
    count: usize,
    collapsed: bool,
    entry_indices: Vec<usize>,
}

pub struct BacklinksState {
    pub target_id: String,
    pub target_label: String,
    pub backlinks: Vec<Backlink>,
    groups: Vec<FileGroup>,
    pub selected: usize,
    pub scroll_offset: usize,
    pub context_lines: usize,
    jump_result: Option<(String, usize)>,
}

impl BacklinksState {
    pub fn new() -> Self {
        Self {
            target_id: String::new(),
            target_label: String::new(),
            backlinks: Vec::new(),
            groups: Vec::new(),
            selected: 0,
            scroll_offset: 0,
            context_lines: 1,
            jump_result: None,
        }
    }

    pub fn set_target(&mut self, id: &str, label: &str, source: &dyn BacklinkSource) {
        self.target_id = id.to_string();
        self.target_label = label.to_string();
        self.backlinks = source.backlinks(id);
        self.rebuild_groups();
        self.selected = 0;
        self.scroll_offset = 0;
    }

    pub fn take_jump(&mut self) -> Option<(String, usize)> {
        self.jump_result.take()
    }

    pub fn total_count(&self) -> usize {
        self.backlinks.len()
    }

    fn rebuild_groups(&mut self) {
        let mut file_map: HashMap<String, Vec<usize>> = HashMap::new();
        for (i, bl) in self.backlinks.iter().enumerate() {
            file_map.entry(bl.source_file.clone()).or_default().push(i);
        }
        self.groups = file_map
            .into_iter()
            .map(|(file, indices)| FileGroup {
                count: indices.len(),
                file,
                collapsed: false,
                entry_indices: indices,
            })
            .collect();
        self.groups.sort_by(|a, b| a.file.cmp(&b.file));
    }

    /// Flat list of visible rows: (group_idx, Option<entry_idx>).
    /// None entry_idx = group header.
    fn visible_rows(&self) -> Vec<(usize, Option<usize>)> {
        let mut rows = Vec::new();
        for (gi, group) in self.groups.iter().enumerate() {
            rows.push((gi, None));
            if !group.collapsed {
                for &ei in &group.entry_indices {
                    rows.push((gi, Some(ei)));
                }
            }
        }
        rows
    }
}

impl Default for BacklinksState {
    fn default() -> Self {
        Self::new()
    }
}

// ── Actions ──────────────────────────────────────────────────────────────────

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    SelectNext,
    SelectPrev,
    ToggleGroup,
    Jump,
    ScrollUp,
    ScrollDown,
}

pub fn handle_action(state: &mut BacklinksState, action: Action) {
    let rows = state.visible_rows();
    match action {
        Action::SelectNext => {
            state.selected = (state.selected + 1).min(rows.len().saturating_sub(1));
        }
        Action::SelectPrev => {
            state.selected = state.selected.saturating_sub(1);
        }
        Action::ToggleGroup => {
            if let Some(&(gi, entry)) = rows.get(state.selected) {
                if entry.is_none() {
                    state.groups[gi].collapsed = !state.groups[gi].collapsed;
                }
            }
        }
        Action::Jump => {
            if let Some(&(_, Some(ei))) = rows.get(state.selected) {
                if let Some(bl) = state.backlinks.get(ei) {
                    state.jump_result = Some((bl.source_file.clone(), bl.source_line));
                }
            }
        }
        Action::ScrollUp => {
            state.scroll_offset = state.scroll_offset.saturating_sub(1);
        }
        Action::ScrollDown => {
            state.scroll_offset += 1;
        }
    }
}

// ── Style ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct BacklinksStyle {
    pub file_header: Style,
    pub line_number: Style,
    pub context_dimmed: Style,
    pub link_highlight: Style,
    pub selected: Style,
    pub collapse_indicator: Style,
    pub title: Style,
    pub body: Style,
}

impl Default for BacklinksStyle {
    fn default() -> Self {
        Self {
            file_header: Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
            line_number: Style::default().fg(Color::DarkGray),
            context_dimmed: Style::default().fg(Color::Rgb(100, 100, 100)),
            link_highlight: Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::UNDERLINED),
            selected: Style::default().bg(Color::Rgb(40, 40, 60)),
            collapse_indicator: Style::default().fg(Color::DarkGray),
            title: Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
            body: Style::default(),
        }
    }
}

// ── Widget ───────────────────────────────────────────────────────────────────

pub struct BacklinksPanel<'a> {
    style: BacklinksStyle,
    block: Option<Block<'a>>,
}

impl<'a> BacklinksPanel<'a> {
    pub fn new(style: BacklinksStyle) -> Self {
        Self { style, block: None }
    }
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }
}

impl StatefulWidget for BacklinksPanel<'_> {
    type State = BacklinksState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let title = format!(
            "Backlinks: {} ({})",
            state.target_label,
            state.total_count()
        );
        let block = self.block.unwrap_or_else(|| Block::default()).title(title);
        let inner = block.inner(area);
        block.render(area, buf);

        if inner.width == 0 || inner.height == 0 {
            return;
        }

        let rows = state.visible_rows();
        let visible_height = inner.height as usize;

        // Adjust scroll
        if state.selected < state.scroll_offset {
            state.scroll_offset = state.selected;
        }
        if state.selected >= state.scroll_offset + visible_height {
            state.scroll_offset = state.selected.saturating_sub(visible_height - 1);
        }

        let display =
            &rows[state.scroll_offset..rows.len().min(state.scroll_offset + visible_height)];

        for (row_i, &(gi, entry)) in display.iter().enumerate() {
            let y = inner.y + row_i as u16;
            let is_selected = state.scroll_offset + row_i == state.selected;

            if is_selected {
                for x in inner.x..inner.x + inner.width {
                    buf[(x, y)].set_style(self.style.selected);
                }
            }

            match entry {
                None => {
                    // File group header
                    let group = &state.groups[gi];
                    let indicator = if group.collapsed { "▶" } else { "▼" };
                    let line = Line::from(vec![
                        Span::styled(format!("{} ", indicator), self.style.collapse_indicator),
                        Span::styled(
                            format!("{} ({})", group.file, group.count),
                            self.style.file_header,
                        ),
                    ]);
                    buf.set_line(inner.x, y, &line, inner.width);
                }
                Some(ei) => {
                    if let Some(bl) = state.backlinks.get(ei) {
                        // Context line with link highlighted
                        let line_num =
                            Span::styled(format!("{:4} ", bl.source_line), self.style.line_number);
                        let context = highlight_link(&bl.context_line, &bl.link_text, &self.style);
                        let mut spans = vec![Span::raw("  "), line_num];
                        spans.extend(context);
                        buf.set_line(inner.x, y, &Line::from(spans), inner.width);
                    }
                }
            }
        }
    }
}

fn highlight_link(line: &str, link_text: &str, style: &BacklinksStyle) -> Vec<Span<'static>> {
    if let Some(pos) = line.find(link_text) {
        vec![
            Span::styled(line[..pos].to_string(), style.body),
            Span::styled(link_text.to_string(), style.link_highlight),
            Span::styled(line[pos + link_text.len()..].to_string(), style.body),
        ]
    } else {
        vec![Span::styled(line.to_string(), style.body)]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_backlinks() -> Vec<Backlink> {
        vec![
            Backlink {
                source_file: "notes.org".into(),
                source_heading: Some("Meeting".into()),
                source_line: 42,
                context_before: "previous".into(),
                context_line: "see [[Ship docs]] for details".into(),
                context_after: "next line".into(),
                link_text: "Ship docs".into(),
            },
            Backlink {
                source_file: "notes.org".into(),
                source_heading: None,
                source_line: 88,
                context_before: "".into(),
                context_line: "related to [[Ship docs]]".into(),
                context_after: "".into(),
                link_text: "Ship docs".into(),
            },
            Backlink {
                source_file: "projects.org".into(),
                source_heading: None,
                source_line: 10,
                context_before: "".into(),
                context_line: "depends on [[Ship docs]]".into(),
                context_after: "".into(),
                link_text: "Ship docs".into(),
            },
        ]
    }

    #[test]
    fn groups_by_file() {
        let bls = sample_backlinks();
        let mut state = BacklinksState::new();
        state.set_target("id-1", "Ship docs", &bls);
        assert_eq!(state.groups.len(), 2); // notes.org, projects.org
    }

    #[test]
    fn collapse_expand() {
        let bls = sample_backlinks();
        let mut state = BacklinksState::new();
        state.set_target("id-1", "Ship docs", &bls);

        let rows_before = state.visible_rows().len();
        // Select first group header, toggle collapse
        state.selected = 0;
        handle_action(&mut state, Action::ToggleGroup);
        let rows_after = state.visible_rows().len();
        assert!(
            rows_after < rows_before,
            "collapsing should reduce visible rows"
        );
    }

    #[test]
    fn jump_on_entry() {
        let bls = sample_backlinks();
        let mut state = BacklinksState::new();
        state.set_target("id-1", "Ship docs", &bls);

        // Navigate to first entry (row 1 = first entry under first group)
        state.selected = 1;
        handle_action(&mut state, Action::Jump);
        let (file, line) = state.take_jump().unwrap();
        assert_eq!(file, "notes.org");
        assert_eq!(line, 42);
    }
}
