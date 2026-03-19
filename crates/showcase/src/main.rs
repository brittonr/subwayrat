//! Interactive showcase of all subwayrat TUI widgets.
//!
//! Navigate between demos with Tab/Shift+Tab. Press q to quit.

use std::io;
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use crossterm::execute;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};
use ratatui::{Frame, Terminal};

use rat_widgets::{
    ConfirmDialog, GridItem, GridSelect, Loader, Notification, ProgressBar,
    ScrollableList, SelectList, Slider, TabBar, TextInput, TreeNode, TreeView, WidgetTheme,
};

use rat_editor::Editor;
use rat_markdown::{MarkdownStyle, PlainHighlighter, render_markdown};
use rat_table::{DataTable, DataTableStyle};

// ── Demo tabs ────────────────────────────────────────────────────────────────

const TABS: &[&str] = &[
    "Widgets",
    "Inputs",
    "Table",
    "Markdown",
    "Editor",
    "Dialogs",
    "Misc",
];

// ── App state ────────────────────────────────────────────────────────────────

struct App {
    tab_bar: TabBar,
    theme: WidgetTheme,
    running: bool,
    tick: u64,

    // Widgets tab
    progress: f64,
    progress_dir: f64,
    slider_val: f64,
    loader: Loader,
    scrollable: ScrollableList,
    notifications: Vec<Notification>,

    // Inputs tab
    text_input: TextInput,
    select_list: SelectList,
    grid: GridSelect,

    // Table tab
    data_table: DataTable,

    // Editor tab
    editor: Editor,

    // Dialogs tab
    confirm: ConfirmDialog,
    input_dialog: rat_widgets::InputDialog,
    tree_view: TreeView,

    // Misc tab
    last_key: String,
}

impl App {
    fn new() -> Self {
        let theme = WidgetTheme {
            primary: Color::Rgb(100, 149, 237),    // cornflower blue
            secondary: Color::Rgb(144, 238, 144),   // light green
            success: Color::Rgb(80, 200, 120),
            warning: Color::Rgb(255, 193, 7),
            error: Color::Rgb(220, 53, 69),
            text: Color::Rgb(220, 220, 220),
            text_muted: Color::Rgb(140, 140, 140),
            text_disabled: Color::Rgb(80, 80, 80),
            border_focused: Color::Rgb(100, 149, 237),
            border_normal: Color::Rgb(60, 60, 60),
            background: Color::Reset,
        };

        let scrollable = ScrollableList::new(
            (1..=30)
                .map(|i| format!("  Item #{i:02} — sample list entry"))
                .collect(),
        )
        .with_highlight_symbol("▸ ")
        .with_highlight_style(
            Style::default()
                .fg(Color::Rgb(100, 149, 237))
                .add_modifier(Modifier::BOLD),
        )
        .with_normal_style(Style::default().fg(Color::Rgb(160, 160, 160)));

        let select_list = SelectList::new(
            "Pick a Color",
            vec![
                "Red".into(),
                "Green".into(),
                "Blue".into(),
                "Yellow".into(),
                "Magenta".into(),
                "Cyan".into(),
            ],
        );

        let grid_items = vec![
            GridItem::new("Rose").with_color(Color::Rgb(255, 100, 100)),
            GridItem::new("Lime").with_color(Color::Rgb(100, 255, 100)),
            GridItem::new("Sky").with_color(Color::Rgb(100, 100, 255)),
            GridItem::new("Gold").with_color(Color::Rgb(255, 215, 0)),
            GridItem::new("Plum").with_color(Color::Rgb(180, 100, 200)),
            GridItem::new("Teal").with_color(Color::Rgb(0, 200, 180)),
            GridItem::new("Coral").with_color(Color::Rgb(255, 127, 80)),
            GridItem::new("Slate").with_color(Color::Rgb(112, 128, 144)),
            GridItem::new("Peach").with_color(Color::Rgb(255, 200, 160)),
        ];
        let grid = GridSelect::new("Color Palette", grid_items, 3);

        let data_table = DataTable::new(
            vec![
                "Name".into(),
                "Language".into(),
                "Stars".into(),
                "License".into(),
            ],
            vec![
                vec!["ratatui".into(), "Rust".into(), "12.4k".into(), "MIT".into()],
                vec!["crossterm".into(), "Rust".into(), "3.2k".into(), "MIT".into()],
                vec!["tui-rs".into(), "Rust".into(), "10.1k".into(), "MIT".into()],
                vec!["blessed".into(), "JavaScript".into(), "11.3k".into(), "MIT".into()],
                vec!["textual".into(), "Python".into(), "26.1k".into(), "MIT".into()],
                vec!["bubbletea".into(), "Go".into(), "28.9k".into(), "MIT".into()],
                vec!["charm".into(), "Go".into(), "17.5k".into(), "MIT".into()],
                vec!["ftxui".into(), "C++".into(), "7.8k".into(), "MIT".into()],
            ],
        );

        let text_input = TextInput::new()
            .with_placeholder("Type something here...")
            .with_focused(true)
            .with_focused_border(Color::Rgb(100, 149, 237));

        let mut editor = Editor::new();
        for c in "Hello from rat-editor!\nMulti-line editing works.\nTry arrow keys to navigate.".chars() {
            editor.insert_char(c);
        }

        let confirm = ConfirmDialog::new("Apply changes to the configuration?");

        let input_dialog = rat_widgets::InputDialog::new("Search");

        let tree = vec![
            TreeNode {
                label: "src/".into(),
                id: "src".into(),
                depth: 0,
                expanded: true,
                children: vec![
                    TreeNode {
                        label: "main.rs".into(),
                        id: "main".into(),
                        depth: 1,
                        expanded: false,
                        children: vec![],
                    },
                    TreeNode {
                        label: "lib.rs".into(),
                        id: "lib".into(),
                        depth: 1,
                        expanded: false,
                        children: vec![],
                    },
                    TreeNode {
                        label: "utils/".into(),
                        id: "utils".into(),
                        depth: 1,
                        expanded: true,
                        children: vec![
                            TreeNode {
                                label: "helpers.rs".into(),
                                id: "helpers".into(),
                                depth: 2,
                                expanded: false,
                                children: vec![],
                            },
                        ],
                    },
                ],
            },
            TreeNode {
                label: "Cargo.toml".into(),
                id: "cargo".into(),
                depth: 0,
                expanded: false,
                children: vec![],
            },
            TreeNode {
                label: "README.md".into(),
                id: "readme".into(),
                depth: 0,
                expanded: false,
                children: vec![],
            },
        ];
        let mut tree_view = TreeView::new("File Tree", tree);
        tree_view.visible = true;

        Self {
            tab_bar: TabBar::new(TABS.to_vec())
                .with_active_style(
                    Style::default()
                        .fg(Color::Rgb(100, 149, 237))
                        .add_modifier(Modifier::BOLD),
                )
                .with_inactive_style(Style::default().fg(Color::Rgb(140, 140, 140)))
                .with_border_color(Color::Rgb(100, 149, 237)),
            theme,
            running: true,
            tick: 0,
            progress: 0.0,
            progress_dir: 0.004,
            slider_val: 0.35,
            loader: Loader::new("Loading data..."),
            scrollable,
            notifications: Vec::new(),
            text_input,
            select_list,
            grid,
            data_table,
            editor,
            confirm,
            input_dialog,
            tree_view,
            last_key: String::new(),
        }
    }

    fn tick(&mut self) {
        self.tick += 1;
        self.loader.tick();

        // Bounce the progress bar
        self.progress += self.progress_dir;
        if self.progress >= 1.0 {
            self.progress = 1.0;
            self.progress_dir = -0.004;
        } else if self.progress <= 0.0 {
            self.progress = 0.0;
            self.progress_dir = 0.004;
        }

        // Expire old notifications
        self.notifications.retain(|n| !n.is_expired());
    }

    fn handle_key(&mut self, code: KeyCode, modifiers: KeyModifiers) {
        self.last_key = format!("{code:?}");

        match code {
            KeyCode::Char('q') if modifiers.contains(KeyModifiers::CONTROL) => {
                self.running = false;
            }
            KeyCode::Char('q') => {
                self.running = false;
            }
            KeyCode::Tab if modifiers.contains(KeyModifiers::SHIFT) => {
                self.tab_bar.select_prev();
            }
            KeyCode::Tab => {
                self.tab_bar.select_next();
            }
            KeyCode::BackTab => {
                self.tab_bar.select_prev();
            }
            _ => self.handle_tab_key(code, modifiers),
        }
    }

    fn handle_tab_key(&mut self, code: KeyCode, _modifiers: KeyModifiers) {
        match self.tab_bar.active_index() {
            0 => self.handle_widgets_key(code),
            1 => self.handle_inputs_key(code),
            2 => self.handle_table_key(code),
            4 => self.handle_editor_key(code),
            5 => self.handle_dialogs_key(code),
            _ => {}
        }
    }

    fn handle_widgets_key(&mut self, code: KeyCode) {
        match code {
            KeyCode::Up => self.scrollable.move_up(),
            KeyCode::Down => self.scrollable.move_down(),
            KeyCode::Left => {
                self.slider_val = (self.slider_val - 0.05).max(0.0);
            }
            KeyCode::Right => {
                self.slider_val = (self.slider_val + 0.05).min(1.0);
            }
            KeyCode::Char('n') => {
                let msgs = ["File saved", "Build complete", "3 tests passed"];
                let msg = msgs[self.tick as usize % msgs.len()];
                self.notifications.push(Notification::info(msg));
            }
            KeyCode::Char('w') => {
                self.notifications
                    .push(Notification::warning("Disk space running low"));
            }
            KeyCode::Char('e') => {
                self.notifications
                    .push(Notification::error("Connection refused"));
            }
            _ => {}
        }
    }

    fn handle_inputs_key(&mut self, code: KeyCode) {
        if self.text_input.is_focused() {
            match code {
                KeyCode::Char(c) => self.text_input.type_char(c),
                KeyCode::Backspace => self.text_input.backspace(),
                KeyCode::Delete => self.text_input.delete(),
                KeyCode::Left => self.text_input.move_left(),
                KeyCode::Right => self.text_input.move_right(),
                KeyCode::Home => self.text_input.move_home(),
                KeyCode::End => self.text_input.move_end(),
                _ => {}
            }
        }
    }

    fn handle_table_key(&mut self, code: KeyCode) {
        match code {
            KeyCode::Up => self.data_table.select_prev(),
            KeyCode::Down => self.data_table.select_next(),
            KeyCode::Left => self.data_table.scroll_left(),
            KeyCode::Right => self.data_table.scroll_right(),
            KeyCode::Home => self.data_table.select_first(),
            KeyCode::End => self.data_table.select_last(),
            _ => {}
        }
    }

    fn handle_editor_key(&mut self, code: KeyCode) {
        match code {
            KeyCode::Char(c) => self.editor.insert_char(c),
            KeyCode::Enter => self.editor.insert_char('\n'),
            KeyCode::Backspace => self.editor.delete_back(),
            KeyCode::Delete => self.editor.delete_forward(),
            KeyCode::Left => self.editor.move_left(),
            KeyCode::Right => self.editor.move_right(),
            KeyCode::Up => self.editor.history_up(),
            KeyCode::Down => self.editor.history_down(),
            KeyCode::Home => self.editor.move_home(),
            KeyCode::End => self.editor.move_end(),
            _ => {}
        }
    }

    fn handle_dialogs_key(&mut self, code: KeyCode) {
        match code {
            KeyCode::Left | KeyCode::Right => self.confirm.toggle(),
            KeyCode::Up => self.tree_view.move_up(),
            KeyCode::Down => self.tree_view.move_down(),
            KeyCode::Char(c) => self.input_dialog.type_char(c),
            KeyCode::Backspace => self.input_dialog.backspace(),
            _ => {}
        }
    }
}

// ── Drawing ──────────────────────────────────────────────────────────────────

fn draw(frame: &mut Frame, app: &mut App) {
    let size = frame.area();

    // Background
    frame.render_widget(Clear, size);

    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // title
            Constraint::Length(1), // tab bar
            Constraint::Min(10),  // content
            Constraint::Length(1), // status bar
        ])
        .split(size);

    // Title bar
    let title = Line::from(vec![
        Span::styled(
            " 🐀 subwayrat ",
            Style::default()
                .fg(Color::Rgb(100, 149, 237))
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "widget showcase",
            Style::default().fg(Color::Rgb(140, 140, 140)),
        ),
    ]);
    frame.render_widget(Paragraph::new(title), outer[0]);

    // Tab bar
    app.tab_bar.render(frame, outer[1], None);

    // Content area
    let content_block = Block::default()
        .borders(Borders::TOP)
        .border_style(Style::default().fg(Color::Rgb(60, 60, 60)));
    let content_area = content_block.inner(outer[2]);
    frame.render_widget(content_block, outer[2]);

    match app.tab_bar.active_index() {
        0 => draw_widgets_tab(frame, content_area, app),
        1 => draw_inputs_tab(frame, content_area, app),
        2 => draw_table_tab(frame, content_area, app),
        3 => draw_markdown_tab(frame, content_area, app),
        4 => draw_editor_tab(frame, content_area, app),
        5 => draw_dialogs_tab(frame, content_area, app),
        6 => draw_misc_tab(frame, content_area, app),
        _ => {}
    }

    // Status bar
    let mut status = Line::from(vec![
        Span::styled(
            " Tab/Shift+Tab",
            Style::default().fg(Color::Rgb(100, 149, 237)),
        ),
        Span::styled(
            " switch sections  ",
            Style::default().fg(Color::Rgb(140, 140, 140)),
        ),
        Span::styled("q", Style::default().fg(Color::Rgb(100, 149, 237))),
        Span::styled(
            " quit  ",
            Style::default().fg(Color::Rgb(140, 140, 140)),
        ),
    ]);
    let last_key_text = format!("last key: {} ", app.last_key);
    status.spans.push(Span::styled(
        last_key_text,
        Style::default().fg(Color::Rgb(80, 80, 80)),
    ));
    frame.render_widget(Paragraph::new(status), outer[3]);

    // Notifications overlay
    for notif in &app.notifications {
        notif.render_themed(frame, size, &app.theme);
    }
}

fn draw_widgets_tab(frame: &mut Frame, area: Rect, app: &mut App) {
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    // Left column
    let left = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // progress bar
            Constraint::Length(3),  // slider
            Constraint::Length(2),  // loader
            Constraint::Min(6),    // scrollable list
        ])
        .split(cols[0]);

    // Progress bar
    let pb_block = Block::default()
        .title(Span::styled(
            " Progress Bar ",
            Style::default()
                .fg(Color::Rgb(100, 149, 237))
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(60, 60, 60)));
    let pb_inner = pb_block.inner(left[0]);
    frame.render_widget(pb_block, left[0]);
    let elapsed = (app.progress * 180.0) as u64;
    let pb = ProgressBar::new(app.progress)
        .with_time_labels(elapsed, 180)
        .with_percentage(true)
        .with_filled_style(Style::default().fg(Color::Rgb(100, 149, 237)))
        .with_empty_style(Style::default().fg(Color::Rgb(60, 60, 60)));
    pb.render(frame, pb_inner);

    // Slider
    let sl_block = Block::default()
        .title(Span::styled(
            " Slider (←/→) ",
            Style::default()
                .fg(Color::Rgb(100, 149, 237))
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(60, 60, 60)));
    let sl_inner = sl_block.inner(left[1]);
    frame.render_widget(sl_block, left[1]);
    let sl = Slider::new(app.slider_val)
        .with_left_label("Vol")
        .with_right_label(format!("{:.0}%", app.slider_val * 100.0))
        .with_filled_style(Style::default().fg(Color::Rgb(144, 238, 144)))
        .with_thumb_style(Style::default().fg(Color::White));
    sl.render(frame, sl_inner);

    // Loader
    let ld_block = Block::default()
        .title(Span::styled(
            " Loader ",
            Style::default()
                .fg(Color::Rgb(100, 149, 237))
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::LEFT | Borders::RIGHT)
        .border_style(Style::default().fg(Color::Rgb(60, 60, 60)));
    let ld_inner = ld_block.inner(left[2]);
    frame.render_widget(ld_block, left[2]);
    app.loader.render_themed(frame, ld_inner, &app.theme);

    // Scrollable list
    let list_block = Block::default()
        .title(Span::styled(
            " ScrollableList (↑/↓) ",
            Style::default()
                .fg(Color::Rgb(100, 149, 237))
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(60, 60, 60)));
    app.scrollable.render(frame, left[3], Some(list_block));

    // Right column: notifications help + grid
    let right = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8), // help text
            Constraint::Min(6),   // grid
        ])
        .split(cols[1]);

    let help_block = Block::default()
        .title(Span::styled(
            " Notifications ",
            Style::default()
                .fg(Color::Rgb(100, 149, 237))
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(60, 60, 60)));
    let help_inner = help_block.inner(right[0]);
    frame.render_widget(help_block, right[0]);
    let help = Paragraph::new(vec![
        Line::from(vec![
            Span::styled("n", Style::default().fg(Color::Rgb(100, 149, 237))),
            Span::styled("  info notification", Style::default().fg(Color::Rgb(160, 160, 160))),
        ]),
        Line::from(vec![
            Span::styled("w", Style::default().fg(Color::Rgb(255, 193, 7))),
            Span::styled("  warning notification", Style::default().fg(Color::Rgb(160, 160, 160))),
        ]),
        Line::from(vec![
            Span::styled("e", Style::default().fg(Color::Rgb(220, 53, 69))),
            Span::styled("  error notification", Style::default().fg(Color::Rgb(160, 160, 160))),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "Notifications auto-dismiss after a few seconds.",
            Style::default().fg(Color::Rgb(100, 100, 100)),
        )),
    ]);
    frame.render_widget(help, help_inner);

    // Grid select
    app.grid.render_themed(frame, right[1], &app.theme);
}

fn draw_inputs_tab(frame: &mut Frame, area: Rect, app: &mut App) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // text input
            Constraint::Length(2),  // spacer + label
            Constraint::Min(8),    // select list display
        ])
        .split(area);

    // Text input
    let input_block = Block::default()
        .title(Span::styled(
            " TextInput ",
            Style::default()
                .fg(Color::Rgb(100, 149, 237))
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::NONE);
    let inner = input_block.inner(rows[0]);
    frame.render_widget(input_block, rows[0]);
    app.text_input.render(frame, inner, None);

    // Label
    let label = Paragraph::new(Line::from(vec![
        Span::styled(
            " Current value: ",
            Style::default().fg(Color::Rgb(140, 140, 140)),
        ),
        Span::styled(
            if app.text_input.value().is_empty() {
                "(empty)"
            } else {
                app.text_input.value()
            },
            Style::default()
                .fg(Color::Rgb(100, 149, 237))
                .add_modifier(Modifier::BOLD),
        ),
    ]));
    frame.render_widget(label, rows[1]);

    // SelectList as centered popup
    app.select_list.render_themed(frame, rows[2], &app.theme);
}

fn draw_table_tab(frame: &mut Frame, area: Rect, app: &mut App) {
    let block = Block::default()
        .title(Span::styled(
            " DataTable (↑/↓ navigate, ←/→ scroll columns) ",
            Style::default()
                .fg(Color::Rgb(100, 149, 237))
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(60, 60, 60)));

    let table_style = DataTableStyle {
        header_style: Style::default()
            .fg(Color::Rgb(100, 149, 237))
            .add_modifier(Modifier::BOLD),
        selected_style: Style::default()
            .bg(Color::Rgb(40, 50, 70))
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
        normal_style: Style::default().fg(Color::Rgb(180, 180, 180)),
        truncation_suffix: "…".into(),
        column_spacing: 2,
    };

    let info = app.data_table.info();
    let info_text = format!(
        " {} rows × {} cols | row {} ",
        info.row_count,
        info.column_count,
        app.data_table.selected_index() + 1,
    );

    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(5), Constraint::Length(1)])
        .split(area);

    app.data_table.render(frame, outer[0], Some(block), &table_style);

    let info_line = Paragraph::new(Line::from(Span::styled(
        info_text,
        Style::default().fg(Color::Rgb(100, 100, 100)),
    )));
    frame.render_widget(info_line, outer[1]);
}

fn draw_markdown_tab(frame: &mut Frame, area: Rect, _app: &App) {
    let md_text = r#"# rat-markdown

Renders **markdown** into styled ratatui `Span`s.

## Features

- **Bold**, *italic*, and ***bold italic***
- `inline code` with background
- ~~strikethrough~~ text
- [Links](https://example.com) are underlined
- Ordered and unordered lists

### Code blocks

```rust
fn main() {
    println!("Hello from a code block!");
}
```

> Blockquotes render with a vertical bar.

### Lists

1. First ordered item
2. Second ordered item
3. Third ordered item

- Unordered item A
- Unordered item B
  - Nested item

---

That horizontal rule above is `---` in source."#;

    let md_style = MarkdownStyle::from_base(Style::default().fg(Color::Rgb(200, 200, 200)));
    let lines = render_markdown(md_text, &md_style, &PlainHighlighter);

    let block = Block::default()
        .title(Span::styled(
            " Markdown Renderer ",
            Style::default()
                .fg(Color::Rgb(100, 149, 237))
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(60, 60, 60)));

    let inner = block.inner(area);
    frame.render_widget(block, area);
    frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), inner);
}

fn draw_editor_tab(frame: &mut Frame, area: Rect, app: &App) {
    let block = Block::default()
        .title(Span::styled(
            " Editor (type to edit, arrows to move) ",
            Style::default()
                .fg(Color::Rgb(100, 149, 237))
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(60, 60, 60)));
    let inner = block.inner(area);
    frame.render_widget(block, area);
    rat_editor::render_editor(frame, &app.editor, inner, "> ", Color::Rgb(100, 149, 237), "Editor");
}

fn draw_dialogs_tab(frame: &mut Frame, area: Rect, app: &App) {
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(33),
            Constraint::Percentage(34),
            Constraint::Percentage(33),
        ])
        .split(area);

    // Confirm dialog
    let confirm_block = Block::default()
        .title(Span::styled(
            " ConfirmDialog (←/→) ",
            Style::default()
                .fg(Color::Rgb(100, 149, 237))
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(60, 60, 60)));
    frame.render_widget(confirm_block, cols[0]);
    app.confirm.render_themed(frame, cols[0], &app.theme);

    // Input dialog
    let input_block = Block::default()
        .title(Span::styled(
            " InputDialog (type) ",
            Style::default()
                .fg(Color::Rgb(100, 149, 237))
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(60, 60, 60)));
    frame.render_widget(input_block, cols[1]);
    app.input_dialog.render_themed(frame, cols[1], &app.theme);

    // Tree view
    let tree_block = Block::default()
        .title(Span::styled(
            " TreeView (↑/↓) ",
            Style::default()
                .fg(Color::Rgb(100, 149, 237))
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(60, 60, 60)));
    frame.render_widget(tree_block, cols[2]);
    app.tree_view.render_themed(frame, cols[2], &app.theme);
}

fn draw_misc_tab(frame: &mut Frame, area: Rect, app: &App) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(12), // crate overview
            Constraint::Min(4),    // theme colors
        ])
        .split(area);

    // Crate overview
    let crate_block = Block::default()
        .title(Span::styled(
            " subwayrat crates ",
            Style::default()
                .fg(Color::Rgb(100, 149, 237))
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(60, 60, 60)));
    let crate_inner = crate_block.inner(rows[0]);
    frame.render_widget(crate_block, rows[0]);

    let crates_text = vec![
        Line::from(vec![
            Span::styled("rat-widgets     ", Style::default().fg(Color::Rgb(100, 149, 237)).add_modifier(Modifier::BOLD)),
            Span::styled("Dialogs, spinner, notifications, tree view, scroll, sliders, tabs", Style::default().fg(Color::Rgb(160, 160, 160))),
        ]),
        Line::from(vec![
            Span::styled("rat-editor      ", Style::default().fg(Color::Rgb(144, 238, 144)).add_modifier(Modifier::BOLD)),
            Span::styled("Multi-line text editor with history", Style::default().fg(Color::Rgb(160, 160, 160))),
        ]),
        Line::from(vec![
            Span::styled("rat-table       ", Style::default().fg(Color::Rgb(255, 193, 7)).add_modifier(Modifier::BOLD)),
            Span::styled("Scrollable data table with auto column sizing", Style::default().fg(Color::Rgb(160, 160, 160))),
        ]),
        Line::from(vec![
            Span::styled("rat-markdown    ", Style::default().fg(Color::Rgb(220, 160, 255)).add_modifier(Modifier::BOLD)),
            Span::styled("Markdown to ratatui Spans with syntax highlighting", Style::default().fg(Color::Rgb(160, 160, 160))),
        ]),
        Line::from(vec![
            Span::styled("rat-diff        ", Style::default().fg(Color::Rgb(255, 127, 80)).add_modifier(Modifier::BOLD)),
            Span::styled("In-process unified diff viewer", Style::default().fg(Color::Rgb(160, 160, 160))),
        ]),
        Line::from(vec![
            Span::styled("rat-streaming   ", Style::default().fg(Color::Rgb(0, 200, 180)).add_modifier(Modifier::BOLD)),
            Span::styled("Streaming output buffer with head/tail truncation", Style::default().fg(Color::Rgb(160, 160, 160))),
        ]),
        Line::from(vec![
            Span::styled("rat-canvas      ", Style::default().fg(Color::Rgb(255, 100, 100)).add_modifier(Modifier::BOLD)),
            Span::styled("Infinite canvas viewport with pan/zoom", Style::default().fg(Color::Rgb(160, 160, 160))),
        ]),
        Line::from(vec![
            Span::styled("rat-nodegraph   ", Style::default().fg(Color::Rgb(112, 128, 144)).add_modifier(Modifier::BOLD)),
            Span::styled("Node-based graph editor with typed ports and auto-layout", Style::default().fg(Color::Rgb(160, 160, 160))),
        ]),
        Line::from(vec![
            Span::styled("rat-spreadsheet ", Style::default().fg(Color::Rgb(255, 200, 160)).add_modifier(Modifier::BOLD)),
            Span::styled("Editable spreadsheet with formulas and cell navigation", Style::default().fg(Color::Rgb(160, 160, 160))),
        ]),
        Line::from(vec![
            Span::styled("+ 6 more        ", Style::default().fg(Color::Rgb(80, 80, 80)).add_modifier(Modifier::BOLD)),
            Span::styled("keymap, leaderkey, branches, layers, selection, image", Style::default().fg(Color::Rgb(100, 100, 100))),
        ]),
    ];
    frame.render_widget(Paragraph::new(crates_text), crate_inner);

    // Theme colors
    let theme_block = Block::default()
        .title(Span::styled(
            " WidgetTheme Colors ",
            Style::default()
                .fg(Color::Rgb(100, 149, 237))
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(60, 60, 60)));
    let theme_inner = theme_block.inner(rows[1]);
    frame.render_widget(theme_block, rows[1]);

    let swatch = |label: &str, color: Color| -> Vec<Span<'static>> {
        vec![
            Span::styled("██ ", Style::default().fg(color)),
            Span::styled(
                format!("{label:<16}"),
                Style::default().fg(Color::Rgb(180, 180, 180)),
            ),
        ]
    };

    let t = &app.theme;
    let color_lines = vec![
        Line::from(
            [
                swatch("primary", t.primary),
                swatch("secondary", t.secondary),
                swatch("success", t.success),
            ]
            .concat(),
        ),
        Line::from(
            [
                swatch("warning", t.warning),
                swatch("error", t.error),
                swatch("text", t.text),
            ]
            .concat(),
        ),
        Line::from(
            [
                swatch("text_muted", t.text_muted),
                swatch("text_disabled", t.text_disabled),
                swatch("border_focused", t.border_focused),
            ]
            .concat(),
        ),
    ];
    frame.render_widget(Paragraph::new(color_lines), theme_inner);
}

// ── Main ─────────────────────────────────────────────────────────────────────

fn main() -> io::Result<()> {
    enable_raw_mode()?;
    execute!(io::stdout(), EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();
    let tick_rate = Duration::from_millis(50);

    while app.running {
        terminal.draw(|f| draw(f, &mut app))?;

        let timeout = tick_rate;
        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == event::KeyEventKind::Press {
                    app.handle_key(key.code, key.modifiers);
                }
            }
        }

        app.tick();
    }

    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen)?;

    Ok(())
}
