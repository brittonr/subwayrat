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
use rat_tree::{Tree as RatTree, TreeState as RatTreeState, TreeStyle as RatTreeStyle, TreeData, SimpleTree, TreeAction, default_keymap};
use rat_keymap::Keymap;

// ── Demo tabs ────────────────────────────────────────────────────────────────

const TABS: &[&str] = &[
    "Widgets",
    "Inputs",
    "Table",
    "Markdown",
    "Editor",
    "Dialogs",
    "Tree",
    "Misc",
];

// ── App state ────────────────────────────────────────────────────────────────

struct App {
    tab_bar: TabBar,
    theme: WidgetTheme,
    running: bool,
    tick: u64,
    focus: usize, // which widget within the current tab has focus

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

    // Tree tab
    rat_tree_data: SimpleTree,
    rat_tree_state: RatTreeState,
    rat_tree_keymap: Keymap<TreeAction, ()>,

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

        let rat_tree_data = SimpleTree::new(vec![
            (0, None, "src/".into()),
            (1, Some(0), "main.rs".into()),
            (2, Some(0), "lib.rs".into()),
            (3, Some(0), "model/".into()),
            (4, Some(3), "tree.rs".into()),
            (5, Some(3), "state.rs".into()),
            (6, Some(0), "render/".into()),
            (7, Some(6), "widget.rs".into()),
            (8, Some(6), "style.rs".into()),
            (9, Some(6), "guides.rs".into()),
            (10, None, "tests/".into()),
            (11, Some(10), "model_test.rs".into()),
            (12, Some(10), "nav_test.rs".into()),
            (13, Some(10), "render_test.rs".into()),
            (14, None, "Cargo.toml".into()),
            (15, None, "README.md".into()),
            (16, None, "LICENSE".into()),
        ]);
        let mut rat_tree_state = RatTreeState::new(&rat_tree_data);
        // Expand root nodes so the tree looks interesting
        rat_tree_state.expanded.insert(0);
        rat_tree_state.expanded.insert(3);
        rat_tree_state.expanded.insert(10);
        rat_tree_state.recompute(&rat_tree_data);
        let rat_tree_keymap = default_keymap();

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
            focus: 0,
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
            rat_tree_data,
            rat_tree_state,
            rat_tree_keymap,
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

    /// How many focusable widgets the current tab has.
    fn focusable_count(&self) -> usize {
        match self.tab_bar.active_index() {
            0 => 3, // slider, scrollable list, grid
            1 => 2, // text input, select list
            2 => 1, // data table
            3 => 0, // markdown (read-only)
            4 => 1, // editor
            5 => 3, // confirm, input dialog, tree view
            6 => 1, // rat-tree
            7 => 0, // misc (read-only)
            _ => 0,
        }
    }

    fn handle_key(&mut self, code: KeyCode, modifiers: KeyModifiers) {
        self.last_key = format!("{code:?}");

        match code {
            KeyCode::Char('q') if modifiers.contains(KeyModifiers::CONTROL) => {
                self.running = false;
            }
            KeyCode::Char('q') => {
                // Only quit if the focused widget doesn't consume chars
                // (text input / input dialog / editor eat 'q')
                let tab = self.tab_bar.active_index();
                let eats_chars = matches!(
                    (tab, self.focus),
                    (1, 0) | (4, 0) | (5, 1)
                );
                if eats_chars {
                    self.handle_tab_key(code, modifiers);
                } else {
                    self.running = false;
                }
            }
            // [ / ] switch between tabs
            KeyCode::Char('[') => {
                self.tab_bar.select_prev();
                self.focus = 0;
            }
            KeyCode::Char(']') => {
                self.tab_bar.select_next();
                self.focus = 0;
            }
            // Tab / Shift+Tab cycle focus within the current tab
            KeyCode::Tab if modifiers.contains(KeyModifiers::SHIFT) => {
                let count = self.focusable_count();
                if count > 0 {
                    self.focus = if self.focus == 0 { count - 1 } else { self.focus - 1 };
                }
            }
            KeyCode::Tab => {
                let count = self.focusable_count();
                if count > 0 {
                    self.focus = (self.focus + 1) % count;
                }
            }
            KeyCode::BackTab => {
                let count = self.focusable_count();
                if count > 0 {
                    self.focus = if self.focus == 0 { count - 1 } else { self.focus - 1 };
                }
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
            6 => self.handle_tree_key(code, _modifiers),
            _ => {}
        }
    }

    fn handle_widgets_key(&mut self, code: KeyCode) {
        // Notification keys work regardless of focus
        match code {
            KeyCode::Char('n') => {
                let msgs = ["File saved", "Build complete", "3 tests passed"];
                let msg = msgs[self.tick as usize % msgs.len()];
                self.notifications.push(Notification::info(msg));
                return;
            }
            KeyCode::Char('w') => {
                self.notifications
                    .push(Notification::warning("Disk space running low"));
                return;
            }
            KeyCode::Char('e') => {
                self.notifications
                    .push(Notification::error("Connection refused"));
                return;
            }
            _ => {}
        }

        match self.focus {
            0 => match code { // slider
                KeyCode::Left => self.slider_val = (self.slider_val - 0.05).max(0.0),
                KeyCode::Right => self.slider_val = (self.slider_val + 0.05).min(1.0),
                _ => {}
            },
            1 => match code { // scrollable list
                KeyCode::Up => self.scrollable.move_up(),
                KeyCode::Down => self.scrollable.move_down(),
                _ => {}
            },
            2 => match code { // grid
                KeyCode::Up => self.grid.move_up(),
                KeyCode::Down => self.grid.move_down(),
                KeyCode::Left => self.grid.move_left(),
                KeyCode::Right => self.grid.move_right(),
                _ => {}
            },
            _ => {}
        }
    }

    fn handle_inputs_key(&mut self, code: KeyCode) {
        match self.focus {
            0 => match code { // text input
                KeyCode::Char(c) => self.text_input.type_char(c),
                KeyCode::Backspace => self.text_input.backspace(),
                KeyCode::Delete => self.text_input.delete(),
                KeyCode::Left => self.text_input.move_left(),
                KeyCode::Right => self.text_input.move_right(),
                KeyCode::Home => self.text_input.move_home(),
                KeyCode::End => self.text_input.move_end(),
                _ => {}
            },
            1 => match code { // select list
                KeyCode::Up => self.select_list.move_up(),
                KeyCode::Down => self.select_list.move_down(),
                _ => {}
            },
            _ => {}
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

    fn handle_tree_key(&mut self, code: KeyCode, modifiers: KeyModifiers) {
        // Convert crossterm 0.28 types to ratatui's re-exported crossterm 0.29
        use ratatui::crossterm::event::{
            KeyCode as RKeyCode, KeyEvent as RKeyEvent, KeyModifiers as RKeyMods,
        };
        let rcode = match code {
            KeyCode::Char(c) => RKeyCode::Char(c),
            KeyCode::Up => RKeyCode::Up,
            KeyCode::Down => RKeyCode::Down,
            KeyCode::Left => RKeyCode::Left,
            KeyCode::Right => RKeyCode::Right,
            KeyCode::Enter => RKeyCode::Enter,
            KeyCode::Esc => RKeyCode::Esc,
            KeyCode::Tab => RKeyCode::Tab,
            KeyCode::BackTab => RKeyCode::BackTab,
            KeyCode::Backspace => RKeyCode::Backspace,
            KeyCode::Delete => RKeyCode::Delete,
            KeyCode::Home => RKeyCode::Home,
            KeyCode::End => RKeyCode::End,
            KeyCode::PageUp => RKeyCode::PageUp,
            KeyCode::PageDown => RKeyCode::PageDown,
            KeyCode::F(n) => RKeyCode::F(n),
            _ => return,
        };
        let mut rmods = RKeyMods::empty();
        if modifiers.contains(KeyModifiers::CONTROL) {
            rmods |= RKeyMods::CONTROL;
        }
        if modifiers.contains(KeyModifiers::SHIFT) {
            rmods |= RKeyMods::SHIFT;
        }
        if modifiers.contains(KeyModifiers::ALT) {
            rmods |= RKeyMods::ALT;
        }
        let event = RKeyEvent::new(rcode, rmods);
        if let Some(action) = self.rat_tree_keymap.resolve(&(), &event) {
            self.rat_tree_state.apply(action, &self.rat_tree_data, 20);
        }
    }

    fn handle_dialogs_key(&mut self, code: KeyCode) {
        match self.focus {
            0 => match code { // confirm dialog
                KeyCode::Left | KeyCode::Right => self.confirm.toggle(),
                KeyCode::Enter => { /* could handle confirm action */ }
                _ => {}
            },
            1 => match code { // input dialog
                KeyCode::Char(c) => self.input_dialog.type_char(c),
                KeyCode::Backspace => self.input_dialog.backspace(),
                _ => {}
            },
            2 => match code { // tree view
                KeyCode::Up => self.tree_view.move_up(),
                KeyCode::Down => self.tree_view.move_down(),
                _ => {}
            },
            _ => {}
        }
    }
}

// ── Drawing ──────────────────────────────────────────────────────────────────

fn border_style(focused: bool) -> Style {
    if focused {
        Style::default().fg(Color::Rgb(100, 149, 237))
    } else {
        Style::default().fg(Color::Rgb(60, 60, 60))
    }
}

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
        6 => draw_tree_tab(frame, content_area, app),
        7 => draw_misc_tab(frame, content_area, app),
        _ => {}
    }

    // Status bar
    let mut status = Line::from(vec![
        Span::styled(
            " [/]",
            Style::default().fg(Color::Rgb(100, 149, 237)),
        ),
        Span::styled(
            " switch tabs  ",
            Style::default().fg(Color::Rgb(140, 140, 140)),
        ),
        Span::styled("Tab", Style::default().fg(Color::Rgb(100, 149, 237))),
        Span::styled(
            " cycle focus  ",
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

    // Progress bar (not focusable — auto-animated)
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

    // Slider — focus 0
    let sl_block = Block::default()
        .title(Span::styled(
            " Slider (←/→) ",
            Style::default()
                .fg(Color::Rgb(100, 149, 237))
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(border_style(app.focus == 0));
    let sl_inner = sl_block.inner(left[1]);
    frame.render_widget(sl_block, left[1]);
    let sl = Slider::new(app.slider_val)
        .with_left_label("Vol")
        .with_right_label(format!("{:.0}%", app.slider_val * 100.0))
        .with_filled_style(Style::default().fg(Color::Rgb(144, 238, 144)))
        .with_thumb_style(Style::default().fg(Color::White));
    sl.render(frame, sl_inner);

    // Loader (not focusable — auto-animated)
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

    // Scrollable list — focus 1
    let list_block = Block::default()
        .title(Span::styled(
            " ScrollableList (↑/↓) ",
            Style::default()
                .fg(Color::Rgb(100, 149, 237))
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(border_style(app.focus == 1));
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

    // Grid select — focus 2
    let grid_block = Block::default()
        .title(Span::styled(
            " GridSelect (arrows) ",
            Style::default()
                .fg(Color::Rgb(100, 149, 237))
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(border_style(app.focus == 2));
    frame.render_widget(grid_block, right[1]);
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

    // Text input — focus 0
    let input_block = Block::default()
        .title(Span::styled(
            " TextInput ",
            Style::default()
                .fg(Color::Rgb(100, 149, 237))
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(border_style(app.focus == 0));
    app.text_input.render(frame, rows[0], Some(input_block));

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

    // SelectList — focus 1
    let select_block = Block::default()
        .title(Span::styled(
            " SelectList (↑/↓) ",
            Style::default()
                .fg(Color::Rgb(100, 149, 237))
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(border_style(app.focus == 1));
    frame.render_widget(select_block, rows[2]);
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

    // Confirm dialog — focus 0
    let confirm_block = Block::default()
        .title(Span::styled(
            " ConfirmDialog (←/→) ",
            Style::default()
                .fg(Color::Rgb(100, 149, 237))
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(border_style(app.focus == 0));
    frame.render_widget(confirm_block, cols[0]);
    app.confirm.render_themed(frame, cols[0], &app.theme);

    // Input dialog — focus 1
    let input_block = Block::default()
        .title(Span::styled(
            " InputDialog (type) ",
            Style::default()
                .fg(Color::Rgb(100, 149, 237))
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(border_style(app.focus == 1));
    frame.render_widget(input_block, cols[1]);
    app.input_dialog.render_themed(frame, cols[1], &app.theme);

    // Tree view — focus 2
    let tree_block = Block::default()
        .title(Span::styled(
            " TreeView (↑/↓) ",
            Style::default()
                .fg(Color::Rgb(100, 149, 237))
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(border_style(app.focus == 2));
    frame.render_widget(tree_block, cols[2]);
    app.tree_view.render_themed(frame, cols[2], &app.theme);
}

fn draw_tree_tab(frame: &mut Frame, area: Rect, app: &mut App) {
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    // Left: the tree widget
    let tree_block = Block::default()
        .title(Span::styled(
            " rat-tree (j/k navigate, l/h expand/collapse, Space toggle) ",
            Style::default()
                .fg(Color::Rgb(100, 149, 237))
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(60, 60, 60)));

    let tree_style = RatTreeStyle::default()
        .with_selected_style(
            Style::default()
                .bg(Color::Rgb(40, 50, 70))
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
        .with_normal_style(Style::default().fg(Color::Rgb(180, 180, 180)));

    let tree_widget = RatTree::new(&app.rat_tree_data)
        .style(tree_style)
        .block(tree_block);

    frame.render_stateful_widget(tree_widget, cols[0], &mut app.rat_tree_state);

    // Right: info panel
    let info = app.rat_tree_state.info();
    let cursor_label = info.cursor_node_id
        .map(|id| app.rat_tree_data.node_label(id).to_string())
        .unwrap_or_else(|| "(none)".into());
    let cursor_depth = info.cursor_depth.map(|d: usize| d.to_string()).unwrap_or_else(|| "-".into());
    let is_leaf = info.cursor_is_leaf.map(|b| if b { "yes" } else { "no" }).unwrap_or("-");

    let info_block = Block::default()
        .title(Span::styled(
            " Tree Info ",
            Style::default()
                .fg(Color::Rgb(100, 149, 237))
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(60, 60, 60)));
    let info_inner = info_block.inner(cols[1]);
    frame.render_widget(info_block, cols[1]);

    let info_lines = vec![
        Line::from(vec![
            Span::styled("Visible rows:  ", Style::default().fg(Color::Rgb(140, 140, 140))),
            Span::styled(format!("{}", info.visible_count), Style::default().fg(Color::Rgb(100, 149, 237))),
        ]),
        Line::from(vec![
            Span::styled("Cursor node:   ", Style::default().fg(Color::Rgb(140, 140, 140))),
            Span::styled(cursor_label, Style::default().fg(Color::Rgb(144, 238, 144))),
        ]),
        Line::from(vec![
            Span::styled("Cursor depth:  ", Style::default().fg(Color::Rgb(140, 140, 140))),
            Span::styled(cursor_depth, Style::default().fg(Color::Rgb(255, 193, 7))),
        ]),
        Line::from(vec![
            Span::styled("Is leaf:       ", Style::default().fg(Color::Rgb(140, 140, 140))),
            Span::styled(is_leaf.to_string(), Style::default().fg(Color::Rgb(220, 160, 255))),
        ]),
        Line::from(""),
        Line::from(Span::styled("Keybindings", Style::default().fg(Color::Rgb(100, 149, 237)).add_modifier(Modifier::BOLD))),
        Line::from(vec![
            Span::styled("j/↓  ", Style::default().fg(Color::Rgb(100, 149, 237))),
            Span::styled("Move down", Style::default().fg(Color::Rgb(160, 160, 160))),
        ]),
        Line::from(vec![
            Span::styled("k/↑  ", Style::default().fg(Color::Rgb(100, 149, 237))),
            Span::styled("Move up", Style::default().fg(Color::Rgb(160, 160, 160))),
        ]),
        Line::from(vec![
            Span::styled("l/→  ", Style::default().fg(Color::Rgb(100, 149, 237))),
            Span::styled("Expand", Style::default().fg(Color::Rgb(160, 160, 160))),
        ]),
        Line::from(vec![
            Span::styled("h/←  ", Style::default().fg(Color::Rgb(100, 149, 237))),
            Span::styled("Collapse", Style::default().fg(Color::Rgb(160, 160, 160))),
        ]),
        Line::from(vec![
            Span::styled("Space", Style::default().fg(Color::Rgb(100, 149, 237))),
            Span::styled(" Toggle", Style::default().fg(Color::Rgb(160, 160, 160))),
        ]),
        Line::from(vec![
            Span::styled("p    ", Style::default().fg(Color::Rgb(100, 149, 237))),
            Span::styled("Go to parent", Style::default().fg(Color::Rgb(160, 160, 160))),
        ]),
        Line::from(vec![
            Span::styled("o    ", Style::default().fg(Color::Rgb(100, 149, 237))),
            Span::styled("First child", Style::default().fg(Color::Rgb(160, 160, 160))),
        ]),
        Line::from(vec![
            Span::styled("J/K  ", Style::default().fg(Color::Rgb(100, 149, 237))),
            Span::styled("Next/prev sibling", Style::default().fg(Color::Rgb(160, 160, 160))),
        ]),
        Line::from(vec![
            Span::styled("g/G  ", Style::default().fg(Color::Rgb(100, 149, 237))),
            Span::styled("First/last row", Style::default().fg(Color::Rgb(160, 160, 160))),
        ]),
    ];
    frame.render_widget(Paragraph::new(info_lines), info_inner);
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
            Span::styled("rat-tree        ", Style::default().fg(Color::Rgb(80, 200, 120)).add_modifier(Modifier::BOLD)),
            Span::styled("Interactive tree navigation with keymap integration", Style::default().fg(Color::Rgb(160, 160, 160))),
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
