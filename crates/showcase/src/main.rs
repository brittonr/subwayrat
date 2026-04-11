//! Interactive showcase of all subwayrat TUI widgets.
//!
//! Navigate between tabs with Ctrl+Left/Right or click them.
//! Cycle focus within a tab with Tab/Shift+Tab.
//! Press Ctrl+q to quit.

use std::io;
use std::time::Duration;

use crossterm::event::{
    self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers, MouseButton,
    MouseEventKind,
};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};
use ratatui::{Frame, Terminal};

use rat_widgets::{
    ConfirmDialog, GridItem, GridSelect, Loader, Notification, ProgressBar, ScrollableList,
    SelectList, Slider, TabBar, TextInput, TreeNode, TreeView, WidgetTheme,
};

use rat_chrome::{OverlayAnchor, OverlayModel, OverlaySize, OverlayStyle, overlay_frame};
use rat_editor::Editor;
use rat_keymap::Keymap;
use rat_markdown::{MarkdownStyle, PlainHighlighter, render_markdown};
use rat_scrolltile::{
    SizeConstraint, Strip, StripConfig, WindowId, compute_layout as tile_compute_layout,
    nav as tile_nav,
};
use rat_spreadsheet::{
    Action as SheetAction, CellAddr, CellValue, Clipboard as SheetClipboard, Spreadsheet,
    SpreadsheetState, SpreadsheetStyle as SheetStyle, handle_action,
};
use rat_table::{DataTable, DataTableStyle};
use rat_tree::{
    SimpleTree, Tree as RatTree, TreeAction, TreeData, TreeState as RatTreeState,
    TreeStyle as RatTreeStyle, default_keymap,
};

use rat_agenda::{
    Action as AgendaAction, Agenda, AgendaItem, AgendaState, AgendaStyle, Date, Time, ViewMode,
    handle_action as agenda_handle_action,
};
use rat_datepicker::{
    CalendarAction, CalendarGrid, CalendarGridState, CalendarStyle, TimeAction, TimeInput,
    TimeInputState, calendar::CalDate, calendar::handle_calendar_action,
    time_input::handle_time_action,
};
use rat_outline::{
    Action as OutlineAction, Outline, OutlineState, OutlineStyle,
    handle_action as outline_handle_action,
};
// rat-fuzzy, rat-capture, rat-backlinks, rat-tags are available as library
// widgets — they work as overlay/popup components composed by the app.

// ── Demo tabs ────────────────────────────────────────────────────────────────

const TABS: &[&str] = &[
    "Widgets",
    "Inputs",
    "Table",
    "Spreadsheet",
    "Markdown",
    "Editor",
    "Dialogs",
    "Tree",
    "Tiler",
    "Outline",
    "Agenda",
    "DatePick",
    "Misc",
];

// ── App state ────────────────────────────────────────────────────────────────

struct App {
    tab_bar: TabBar,
    theme: WidgetTheme,
    running: bool,
    tick: u64,
    focus: usize,

    /// Stored rects for hit-testing mouse clicks.
    tab_bar_area: Rect,
    /// Per-widget areas for the current tab (for mouse focus).
    widget_areas: Vec<Rect>,

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

    // Spreadsheet tab
    sheet_state: SpreadsheetState,
    sheet_clip: SheetClipboard,

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

    // Tiler tab
    tiler_strip: Strip,
    tiler_windows: Vec<(WindowId, String, Color)>,
    tiler_next_panel: usize,

    // Outline tab
    outline_state: OutlineState,

    // Agenda tab
    agenda_state: AgendaState,
    agenda_items: Vec<AgendaItem>,

    // DatePicker tab
    calendar_state: CalendarGridState,
    time_state: TimeInputState,

    // Misc tab
    last_key: String,
}

impl App {
    fn new() -> Self {
        let theme = WidgetTheme {
            primary: Color::Rgb(100, 149, 237),
            secondary: Color::Rgb(144, 238, 144),
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
                vec![
                    "ratatui".into(),
                    "Rust".into(),
                    "12.4k".into(),
                    "MIT".into(),
                ],
                vec![
                    "crossterm".into(),
                    "Rust".into(),
                    "3.2k".into(),
                    "MIT".into(),
                ],
                vec!["tui-rs".into(), "Rust".into(), "10.1k".into(), "MIT".into()],
                vec![
                    "blessed".into(),
                    "JavaScript".into(),
                    "11.3k".into(),
                    "MIT".into(),
                ],
                vec![
                    "textual".into(),
                    "Python".into(),
                    "26.1k".into(),
                    "MIT".into(),
                ],
                vec![
                    "bubbletea".into(),
                    "Go".into(),
                    "28.9k".into(),
                    "MIT".into(),
                ],
                vec!["charm".into(), "Go".into(), "17.5k".into(), "MIT".into()],
                vec!["ftxui".into(), "C++".into(), "7.8k".into(), "MIT".into()],
            ],
        );

        // Spreadsheet with sample data
        let mut sheet_state = SpreadsheetState::new(8, 20);
        let headers = ["Item", "Category", "Q1", "Q2", "Q3", "Q4", "Total", "Avg"];
        for (col, h) in headers.iter().enumerate() {
            sheet_state
                .grid
                .set(CellAddr { col, row: 0 }, CellValue::Text(h.to_string()));
        }
        let sample = [
            ("Widget A", "Hardware", 120.0, 135.0, 142.0, 158.0),
            ("Widget B", "Hardware", 85.0, 92.0, 88.0, 96.0),
            ("Service X", "Software", 200.0, 210.0, 225.0, 240.0),
            ("Service Y", "Software", 150.0, 148.0, 155.0, 162.0),
            ("Part C", "Hardware", 45.0, 50.0, 48.0, 52.0),
        ];
        for (r, (name, cat, q1, q2, q3, q4)) in sample.iter().enumerate() {
            let row = r + 1;
            sheet_state
                .grid
                .set(CellAddr { col: 0, row }, CellValue::Text(name.to_string()));
            sheet_state
                .grid
                .set(CellAddr { col: 1, row }, CellValue::Text(cat.to_string()));
            sheet_state
                .grid
                .set(CellAddr { col: 2, row }, CellValue::Number(*q1));
            sheet_state
                .grid
                .set(CellAddr { col: 3, row }, CellValue::Number(*q2));
            sheet_state
                .grid
                .set(CellAddr { col: 4, row }, CellValue::Number(*q3));
            sheet_state
                .grid
                .set(CellAddr { col: 5, row }, CellValue::Number(*q4));
            sheet_state.grid.set(
                CellAddr { col: 6, row },
                CellValue::Number(q1 + q2 + q3 + q4),
            );
            sheet_state.grid.set(
                CellAddr { col: 7, row },
                CellValue::Number((q1 + q2 + q3 + q4) / 4.0),
            );
        }
        sheet_state.set_col_width(0, 12);
        sheet_state.set_col_width(1, 10);
        sheet_state.frozen_rows = 1;
        let sheet_clip = SheetClipboard::default();

        let text_input = TextInput::new()
            .with_placeholder("Type something here...")
            .with_focused(true)
            .with_focused_border(Color::Rgb(100, 149, 237));

        let mut editor = Editor::new();
        for c in
            "Hello from rat-editor!\nMulti-line editing works.\nTry arrow keys to navigate.".chars()
        {
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
                        children: vec![TreeNode {
                            label: "helpers.rs".into(),
                            id: "helpers".into(),
                            depth: 2,
                            expanded: false,
                            children: vec![],
                        }],
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
        tree_view.model.visible = true;

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
        rat_tree_state.expanded.insert(0);
        rat_tree_state.expanded.insert(3);
        rat_tree_state.expanded.insert(10);
        rat_tree_state.recompute(&rat_tree_data);
        let rat_tree_keymap = default_keymap();

        // Outline demo
        let mut outline_state = OutlineState::new();
        outline_state.load_text(
            "* TODO Project Alpha\nDesign the new widget system.\n\
             ** IN_PROGRESS [#A] Core engine :code:rust:\n\
             Implement the parser and state machine.\n\
             ** TODO [#B] Tests :test:\nWrite property-based tests.\n\
             *** TODO Unit tests\n*** TODO Integration tests\n\
             * DONE Project Beta :archived:\nShipped in v0.9.\n\
             ** DONE Documentation\n** DONE Release notes\n\
             * TODO Backlog\n- Review PRs\n- Update deps\n",
        );

        // Agenda demo
        let today = Date::new(2026, 3, 26);
        let agenda_items = vec![
            AgendaItem {
                id: "1".into(),
                title: "Team standup".into(),
                status: Some("TODO".into()),
                priority: Some('A'),
                tags: vec!["work".into()],
                scheduled: Some(today),
                deadline: None,
                time_start: Some(Time::new(9, 30)),
                time_end: Some(Time::new(9, 45)),
                source_file: None,
                source_line: None,
            },
            AgendaItem {
                id: "2".into(),
                title: "Code review".into(),
                status: Some("TODO".into()),
                priority: Some('B'),
                tags: vec!["work".into()],
                scheduled: Some(today),
                deadline: None,
                time_start: Some(Time::new(14, 0)),
                time_end: None,
                source_file: None,
                source_line: None,
            },
            AgendaItem {
                id: "3".into(),
                title: "Write tests".into(),
                status: Some("IN_PROGRESS".into()),
                priority: None,
                tags: vec!["code".into()],
                scheduled: Some(today),
                deadline: Some(today.add_days(2)),
                time_start: None,
                time_end: None,
                source_file: None,
                source_line: None,
            },
            AgendaItem {
                id: "4".into(),
                title: "Grocery shopping".into(),
                status: Some("TODO".into()),
                priority: Some('C'),
                tags: vec!["personal".into()],
                scheduled: Some(today.next_day()),
                deadline: None,
                time_start: Some(Time::new(18, 0)),
                time_end: None,
                source_file: None,
                source_line: None,
            },
            AgendaItem {
                id: "5".into(),
                title: "Dentist appointment".into(),
                status: Some("TODO".into()),
                priority: Some('A'),
                tags: vec!["personal".into()],
                scheduled: Some(today.add_days(3)),
                deadline: None,
                time_start: Some(Time::new(10, 0)),
                time_end: Some(Time::new(11, 0)),
                source_file: None,
                source_line: None,
            },
        ];
        let mut agenda_state = AgendaState::new(today);
        agenda_state.refresh(&agenda_items);

        // DatePicker demo
        let cal_today = CalDate::new(2026, 3, 26);
        let calendar_state = CalendarGridState::new(cal_today);
        let time_state = TimeInputState::new(14, 30);

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
            tab_bar_area: Rect::default(),
            widget_areas: Vec::new(),
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
            sheet_state,
            sheet_clip,
            editor,
            confirm,
            input_dialog,
            tree_view,
            rat_tree_data,
            rat_tree_state,
            rat_tree_keymap,
            tiler_strip: Strip::new(StripConfig::default()),
            tiler_windows: Vec::new(),
            tiler_next_panel: 0,
            outline_state,
            agenda_state,
            agenda_items,
            calendar_state,
            time_state,
            last_key: String::new(),
        }
    }

    fn tick(&mut self) {
        self.tick += 1;
        self.loader.tick();

        self.progress += self.progress_dir;
        if self.progress >= 1.0 {
            self.progress = 1.0;
            self.progress_dir = -0.004;
        } else if self.progress <= 0.0 {
            self.progress = 0.0;
            self.progress_dir = 0.004;
        }

        self.notifications.retain(|n| !n.is_expired());
    }

    fn focusable_count(&self) -> usize {
        match self.tab_bar.active_index() {
            0 => 3,  // slider, scrollable list, grid
            1 => 2,  // text input, select list
            2 => 1,  // data table
            3 => 1,  // spreadsheet
            4 => 0,  // markdown (read-only)
            5 => 1,  // editor
            6 => 3,  // confirm, input dialog, tree view
            7 => 1,  // rat-tree
            8 => 0,  // tiler (has own nav)
            9 => 1,  // outline
            10 => 1, // agenda
            11 => 2, // datepicker (calendar, time)
            12 => 0, // misc (read-only)
            _ => 0,
        }
    }

    fn switch_tab(&mut self, idx: usize) {
        if idx < self.tab_bar.len() {
            while self.tab_bar.active_index() != idx {
                self.tab_bar.select_next();
            }
            self.focus = 0;
        }
    }

    fn handle_key(&mut self, code: KeyCode, modifiers: KeyModifiers) {
        self.last_key = format!("{code:?}");

        // Global: Ctrl+q always quits
        if code == KeyCode::Char('q') && modifiers.contains(KeyModifiers::CONTROL) {
            self.running = false;
            return;
        }

        // Ctrl+Left/Right switch tabs
        if modifiers.contains(KeyModifiers::CONTROL) {
            match code {
                KeyCode::Left => {
                    self.tab_bar.select_prev();
                    self.focus = 0;
                    return;
                }
                KeyCode::Right => {
                    self.tab_bar.select_next();
                    self.focus = 0;
                    return;
                }
                _ => {}
            }
        }

        match code {
            // Tab / Shift+Tab cycle focus within the current tab
            KeyCode::Tab if modifiers.contains(KeyModifiers::SHIFT) => {
                let count = self.focusable_count();
                if count > 1 {
                    self.focus = if self.focus == 0 {
                        count - 1
                    } else {
                        self.focus - 1
                    };
                }
            }
            KeyCode::Tab => {
                let count = self.focusable_count();
                if count > 1 {
                    self.focus = (self.focus + 1) % count;
                }
            }
            KeyCode::BackTab => {
                let count = self.focusable_count();
                if count > 1 {
                    self.focus = if self.focus == 0 {
                        count - 1
                    } else {
                        self.focus - 1
                    };
                }
            }
            _ => self.handle_tab_key(code, modifiers),
        }
    }

    fn handle_tab_key(&mut self, code: KeyCode, modifiers: KeyModifiers) {
        match self.tab_bar.active_index() {
            0 => self.handle_widgets_key(code),
            1 => self.handle_inputs_key(code),
            2 => self.handle_table_key(code),
            3 => self.handle_sheet_key(code, modifiers),
            5 => self.handle_editor_key(code),
            6 => self.handle_dialogs_key(code),
            7 => self.handle_tree_key(code, modifiers),
            8 => self.handle_tiler_key(code),
            9 => self.handle_outline_key(code),
            10 => self.handle_agenda_key(code),
            11 => self.handle_datepicker_key(code),
            _ => {}
        }
    }

    fn handle_widgets_key(&mut self, code: KeyCode) {
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
            0 => match code {
                KeyCode::Left => self.slider_val = (self.slider_val - 0.05).max(0.0),
                KeyCode::Right => self.slider_val = (self.slider_val + 0.05).min(1.0),
                _ => {}
            },
            1 => match code {
                KeyCode::Up => self.scrollable.move_up(),
                KeyCode::Down => self.scrollable.move_down(),
                _ => {}
            },
            2 => match code {
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
            0 => match code {
                KeyCode::Char(c) => self.text_input.type_char(c),
                KeyCode::Backspace => self.text_input.backspace(),
                KeyCode::Delete => self.text_input.delete(),
                KeyCode::Left => self.text_input.move_left(),
                KeyCode::Right => self.text_input.move_right(),
                KeyCode::Home => self.text_input.move_home(),
                KeyCode::End => self.text_input.move_end(),
                _ => {}
            },
            1 => match code {
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

    fn handle_sheet_key(&mut self, code: KeyCode, modifiers: KeyModifiers) {
        let action = match code {
            KeyCode::Up => Some(SheetAction::MoveUp),
            KeyCode::Down => Some(SheetAction::MoveDown),
            KeyCode::Left if !self.sheet_state.edit.editing => Some(SheetAction::MoveLeft),
            KeyCode::Right if !self.sheet_state.edit.editing => Some(SheetAction::MoveRight),
            KeyCode::Left if self.sheet_state.edit.editing => Some(SheetAction::EditCursorLeft),
            KeyCode::Right if self.sheet_state.edit.editing => Some(SheetAction::EditCursorRight),
            KeyCode::Home if modifiers.contains(KeyModifiers::CONTROL) => {
                Some(SheetAction::MoveHomeAll)
            }
            KeyCode::End if modifiers.contains(KeyModifiers::CONTROL) => {
                Some(SheetAction::MoveEndAll)
            }
            KeyCode::Home => Some(SheetAction::MoveHome),
            KeyCode::End => Some(SheetAction::MoveEnd),
            KeyCode::PageUp => Some(SheetAction::PageUp),
            KeyCode::PageDown => Some(SheetAction::PageDown),
            KeyCode::Enter if self.sheet_state.edit.editing => Some(SheetAction::CommitEdit),
            KeyCode::Enter => Some(SheetAction::EnterEdit(None)),
            KeyCode::Esc => Some(SheetAction::CancelEdit),
            KeyCode::Backspace if self.sheet_state.edit.editing => Some(SheetAction::Backspace),
            KeyCode::Delete if self.sheet_state.edit.editing => Some(SheetAction::Delete),
            KeyCode::Delete => Some(SheetAction::DeleteContent),
            KeyCode::Char('z') if modifiers.contains(KeyModifiers::CONTROL) => {
                Some(SheetAction::Undo)
            }
            KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => {
                Some(SheetAction::Copy)
            }
            KeyCode::Char('v') if modifiers.contains(KeyModifiers::CONTROL) => {
                Some(SheetAction::Paste)
            }
            KeyCode::Char(c) => {
                if self.sheet_state.edit.editing {
                    Some(SheetAction::TypeChar(c))
                } else {
                    Some(SheetAction::EnterEdit(Some(c)))
                }
            }
            _ => None,
        };
        if let Some(a) = action {
            handle_action(&mut self.sheet_state, a, &mut self.sheet_clip);
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
            0 => match code {
                KeyCode::Left | KeyCode::Right => self.confirm.toggle(),
                _ => {}
            },
            1 => match code {
                KeyCode::Char(c) => self.input_dialog.type_char(c),
                KeyCode::Backspace => self.input_dialog.backspace(),
                _ => {}
            },
            2 => match code {
                KeyCode::Up => self.tree_view.move_up(),
                KeyCode::Down => self.tree_view.move_down(),
                _ => {}
            },
            _ => {}
        }
    }

    fn handle_tiler_key(&mut self, code: KeyCode) {
        let size = crossterm::terminal::size().unwrap_or((80, 24));
        match code {
            // Navigation.
            KeyCode::Left => tile_nav::focus_left(&mut self.tiler_strip, size.0, size.1),
            KeyCode::Right => tile_nav::focus_right(&mut self.tiler_strip, size.0, size.1),
            KeyCode::Up => tile_nav::focus_up(&mut self.tiler_strip),
            KeyCode::Down => tile_nav::focus_down(&mut self.tiler_strip),
            KeyCode::Home => tile_nav::focus_first(&mut self.tiler_strip),
            KeyCode::End => tile_nav::focus_last(&mut self.tiler_strip),
            // Add panel in new column to the right of focused.
            KeyCode::Char('a') => {
                let col = self
                    .tiler_strip
                    .focused()
                    .and_then(|id| self.tiler_strip.find_window(id))
                    .map(|(c, _)| c + 1)
                    .unwrap_or(self.tiler_strip.column_count());
                self.tiler_add_panel(col, 0);
            }
            // Split: add panel below focused in same column.
            KeyCode::Char('s') => {
                if let Some((col, win)) = self
                    .tiler_strip
                    .focused()
                    .and_then(|id| self.tiler_strip.find_window(id))
                {
                    self.tiler_add_panel(col, win + 1);
                }
            }
            // Remove focused panel.
            KeyCode::Char('x') => {
                if let Some(id) = self.tiler_strip.focused() {
                    self.tiler_windows.retain(|(wid, _, _)| *wid != id);
                    self.tiler_strip.remove_window(id);
                }
            }
            // Widen focused column.
            KeyCode::Char(']') => {
                if let Some((col, _)) = self
                    .tiler_strip
                    .focused()
                    .and_then(|id| self.tiler_strip.find_window(id))
                {
                    let cur = tiler_column_fixed_width(&self.tiler_strip, col);
                    self.tiler_strip
                        .resize_column(col, SizeConstraint::Fixed(cur.saturating_add(4)));
                }
            }
            // Narrow focused column.
            KeyCode::Char('[') => {
                if let Some((col, _)) = self
                    .tiler_strip
                    .focused()
                    .and_then(|id| self.tiler_strip.find_window(id))
                {
                    let cur = tiler_column_fixed_width(&self.tiler_strip, col);
                    self.tiler_strip
                        .resize_column(col, SizeConstraint::Fixed(cur.saturating_sub(4).max(8)));
                }
            }
            _ => {}
        }
    }

    fn tiler_add_panel(&mut self, col: usize, stack_pos: usize) {
        let colors = [
            Color::Rgb(100, 149, 237),
            Color::Rgb(144, 238, 144),
            Color::Rgb(255, 193, 7),
            Color::Rgb(220, 160, 255),
            Color::Rgb(0, 200, 180),
            Color::Rgb(255, 127, 80),
            Color::Rgb(255, 100, 100),
            Color::Rgb(112, 128, 144),
        ];
        self.tiler_next_panel += 1;
        let idx = self.tiler_next_panel;
        let color = colors[idx % colors.len()];
        let name = format!("Panel {idx}");
        let id = self.tiler_strip.insert_window(
            col,
            stack_pos,
            SizeConstraint::default(),
            SizeConstraint::default(),
        );
        // New columns get a fixed width so the strip can grow beyond the viewport.
        if self.tiler_strip.column_count() > 0 {
            self.tiler_strip
                .resize_column(col, SizeConstraint::Fixed(30));
        }
        self.tiler_windows.push((id, name, color));
        self.tiler_strip.focus_set(id);
    }

    fn handle_outline_key(&mut self, code: KeyCode) {
        let action = match code {
            KeyCode::Char(c) => OutlineAction::InsertChar(c),
            KeyCode::Enter => OutlineAction::InsertChar('\n'),
            KeyCode::Backspace => OutlineAction::DeleteBack,
            KeyCode::Delete => OutlineAction::DeleteForward,
            KeyCode::Left => OutlineAction::MoveLeft,
            KeyCode::Right => OutlineAction::MoveRight,
            KeyCode::Up => OutlineAction::MoveUp,
            KeyCode::Down => OutlineAction::MoveDown,
            KeyCode::Home => OutlineAction::MoveHome,
            KeyCode::End => OutlineAction::MoveEnd,
            KeyCode::F(5) => OutlineAction::CycleVisibility,
            KeyCode::F(6) => OutlineAction::CycleTodo,
            KeyCode::F(7) => OutlineAction::Promote,
            KeyCode::F(8) => OutlineAction::Demote,
            KeyCode::F(9) => OutlineAction::FoldAll,
            KeyCode::F(10) => OutlineAction::UnfoldAll,
            KeyCode::F(11) => OutlineAction::MoveSubtreeUp,
            KeyCode::F(12) => OutlineAction::MoveSubtreeDown,
            _ => return,
        };
        outline_handle_action(&mut self.outline_state, action);
    }

    fn handle_agenda_key(&mut self, code: KeyCode) {
        let action = match code {
            KeyCode::Left => AgendaAction::PrevDay,
            KeyCode::Right => AgendaAction::NextDay,
            KeyCode::Up => AgendaAction::SelectPrevItem,
            KeyCode::Down => AgendaAction::SelectNextItem,
            KeyCode::Char('[') => AgendaAction::PrevWeek,
            KeyCode::Char(']') => AgendaAction::NextWeek,
            KeyCode::Char('{') => AgendaAction::PrevMonth,
            KeyCode::Char('}') => AgendaAction::NextMonth,
            KeyCode::Char('d') => AgendaAction::SwitchView(ViewMode::Day),
            KeyCode::Char('w') => AgendaAction::SwitchView(ViewMode::Week),
            KeyCode::Char('m') => AgendaAction::SwitchView(ViewMode::Month),
            _ => return,
        };
        let result = agenda_handle_action(&mut self.agenda_state, action);
        if result == rat_agenda::ActionResult::NeedsRefresh {
            self.agenda_state.refresh(&self.agenda_items);
        }
    }

    fn handle_datepicker_key(&mut self, code: KeyCode) {
        match self.focus {
            0 => {
                let action = match code {
                    KeyCode::Left => CalendarAction::PrevDay,
                    KeyCode::Right => CalendarAction::NextDay,
                    KeyCode::Up => CalendarAction::PrevWeek,
                    KeyCode::Down => CalendarAction::NextWeek,
                    KeyCode::Char('[') => CalendarAction::PrevMonth,
                    KeyCode::Char(']') => CalendarAction::NextMonth,
                    _ => return,
                };
                handle_calendar_action(&mut self.calendar_state, action);
            }
            1 => {
                let action = match code {
                    KeyCode::Up => TimeAction::Increment,
                    KeyCode::Down => TimeAction::Decrement,
                    KeyCode::Left | KeyCode::BackTab => TimeAction::PrevField,
                    KeyCode::Right | KeyCode::Tab => TimeAction::NextField,
                    KeyCode::Char(c) if c.is_ascii_digit() => TimeAction::Digit(c as u8 - b'0'),
                    _ => return,
                };
                handle_time_action(&mut self.time_state, action);
            }
            _ => {}
        }
    }

    fn handle_mouse_click(&mut self, x: u16, y: u16) {
        // Click on tab bar?
        if y >= self.tab_bar_area.y
            && y < self.tab_bar_area.y + self.tab_bar_area.height
            && x >= self.tab_bar_area.x
            && x < self.tab_bar_area.x + self.tab_bar_area.width
        {
            // Approximate which tab was clicked by dividing the bar evenly
            let tab_count = self.tab_bar.len() as u16;
            if tab_count > 0 {
                let rel_x = x.saturating_sub(self.tab_bar_area.x);
                let idx = (rel_x * tab_count / self.tab_bar_area.width) as usize;
                self.switch_tab(idx.min(self.tab_bar.len() - 1));
            }
            return;
        }

        // Click on a widget area to focus it?
        for (i, area) in self.widget_areas.iter().enumerate() {
            if x >= area.x && x < area.x + area.width && y >= area.y && y < area.y + area.height {
                self.focus = i;

                // For spreadsheet, translate click to cell address
                if self.tab_bar.active_index() == 3 {
                    self.handle_sheet_click(x, y, *area);
                }
                return;
            }
        }
    }

    fn handle_sheet_click(&mut self, x: u16, y: u16, area: Rect) {
        // Rough translation: skip row header (4 chars) and column header (1 row)
        let rel_x = x.saturating_sub(area.x + 4) as usize;
        let rel_y = y.saturating_sub(area.y + 1) as usize;
        let col = self.sheet_state.scroll.offset_col
            + rel_x / (self.sheet_state.default_col_width as usize + 1);
        let row = self.sheet_state.scroll.offset_row + rel_y;
        if col < self.sheet_state.grid.col_count() && row < self.sheet_state.grid.row_count() {
            handle_action(
                &mut self.sheet_state,
                SheetAction::ClickCell(CellAddr { col, row }),
                &mut self.sheet_clip,
            );
        }
    }

    fn handle_scroll(&mut self, down: bool) {
        match self.tab_bar.active_index() {
            0 if self.focus == 1 => {
                if down {
                    self.scrollable.move_down();
                } else {
                    self.scrollable.move_up();
                }
            }
            2 => {
                if down {
                    self.data_table.select_next();
                } else {
                    self.data_table.select_prev();
                }
            }
            3 => {
                let action = if down {
                    SheetAction::MoveDown
                } else {
                    SheetAction::MoveUp
                };
                handle_action(&mut self.sheet_state, action, &mut self.sheet_clip);
            }
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
    frame.render_widget(Clear, size);

    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // title
            Constraint::Length(1), // tab bar
            Constraint::Min(10),   // content
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

    // Tab bar — store area for mouse hit-testing
    app.tab_bar_area = outer[1];
    app.tab_bar.render(frame, outer[1], None);

    // Content area
    let content_block = Block::default()
        .borders(Borders::TOP)
        .border_style(Style::default().fg(Color::Rgb(60, 60, 60)));
    let content_area = content_block.inner(outer[2]);
    frame.render_widget(content_block, outer[2]);

    // Clear widget areas before each tab populates them
    app.widget_areas.clear();

    match app.tab_bar.active_index() {
        0 => draw_widgets_tab(frame, content_area, app),
        1 => draw_inputs_tab(frame, content_area, app),
        2 => draw_table_tab(frame, content_area, app),
        3 => draw_sheet_tab(frame, content_area, app),
        4 => draw_markdown_tab(frame, content_area, app),
        5 => draw_editor_tab(frame, content_area, app),
        6 => draw_dialogs_tab(frame, content_area, app),
        7 => draw_tree_tab(frame, content_area, app),
        8 => draw_tiler_tab(frame, content_area, app),
        9 => draw_outline_tab(frame, content_area, app),
        10 => draw_agenda_tab(frame, content_area, app),
        11 => draw_datepicker_tab(frame, content_area, app),
        12 => draw_misc_tab(frame, content_area, app),
        _ => {}
    }

    // Status bar
    let mut status = Line::from(vec![
        Span::styled(" Ctrl+←/→", Style::default().fg(Color::Rgb(100, 149, 237))),
        Span::styled(" tabs  ", Style::default().fg(Color::Rgb(140, 140, 140))),
        Span::styled("Tab", Style::default().fg(Color::Rgb(100, 149, 237))),
        Span::styled(" focus  ", Style::default().fg(Color::Rgb(140, 140, 140))),
        Span::styled("Ctrl+q", Style::default().fg(Color::Rgb(100, 149, 237))),
        Span::styled(" quit  ", Style::default().fg(Color::Rgb(140, 140, 140))),
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

    let left = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(2),
            Constraint::Min(6),
        ])
        .split(cols[0]);

    // Progress bar (auto-animated, not focusable)
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
    app.widget_areas.push(left[1]); // index 0
    let sl = Slider::new(app.slider_val)
        .with_left_label("Vol")
        .with_right_label(format!("{:.0}%", app.slider_val * 100.0))
        .with_filled_style(Style::default().fg(Color::Rgb(144, 238, 144)))
        .with_thumb_style(Style::default().fg(Color::White));
    sl.render(frame, sl_inner);

    // Loader (auto-animated, not focusable)
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
    app.widget_areas.push(left[3]); // index 1
    app.scrollable.render(frame, left[3], Some(list_block));

    // Right column
    let right = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(8), Constraint::Min(6)])
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
            Span::styled(
                "  info notification",
                Style::default().fg(Color::Rgb(160, 160, 160)),
            ),
        ]),
        Line::from(vec![
            Span::styled("w", Style::default().fg(Color::Rgb(255, 193, 7))),
            Span::styled(
                "  warning notification",
                Style::default().fg(Color::Rgb(160, 160, 160)),
            ),
        ]),
        Line::from(vec![
            Span::styled("e", Style::default().fg(Color::Rgb(220, 53, 69))),
            Span::styled(
                "  error notification",
                Style::default().fg(Color::Rgb(160, 160, 160)),
            ),
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
    app.widget_areas.push(right[1]); // index 2
    app.grid.render_themed(frame, right[1], &app.theme);
}

fn draw_inputs_tab(frame: &mut Frame, area: Rect, app: &mut App) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(2),
            Constraint::Min(8),
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
    app.widget_areas.push(rows[0]); // index 0
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
    app.widget_areas.push(rows[2]); // index 1
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

    app.widget_areas.push(outer[0]); // index 0
    app.data_table
        .render(frame, outer[0], Some(block), &table_style);

    let info_line = Paragraph::new(Line::from(Span::styled(
        info_text,
        Style::default().fg(Color::Rgb(100, 100, 100)),
    )));
    frame.render_widget(info_line, outer[1]);
}

fn draw_sheet_tab(frame: &mut Frame, area: Rect, app: &mut App) {
    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(5), Constraint::Length(1)])
        .split(area);

    let block = Block::default()
        .title(Span::styled(
            " Spreadsheet (arrows navigate, Enter edit, Esc cancel) ",
            Style::default()
                .fg(Color::Rgb(100, 149, 237))
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(border_style(true));

    let sheet_style = SheetStyle {
        header_style: Style::default()
            .fg(Color::Rgb(100, 149, 237))
            .add_modifier(Modifier::BOLD),
        cursor_style: Style::default()
            .bg(Color::Rgb(40, 50, 70))
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
        selection_style: Style::default()
            .bg(Color::Rgb(30, 40, 60))
            .fg(Color::Rgb(200, 200, 200)),
        cell_style: Style::default().fg(Color::Rgb(180, 180, 180)),
        edit_style: Style::default()
            .fg(Color::Rgb(255, 255, 200))
            .bg(Color::Rgb(50, 50, 80)),
    };

    let widget = Spreadsheet::new().block(block).style(sheet_style);
    app.widget_areas.push(outer[0]); // index 0
    frame.render_stateful_widget(widget, outer[0], &mut app.sheet_state);

    // Info line
    let pos = app.sheet_state.cursor.position;
    let col_letter = (b'A' + pos.col as u8) as char;
    let editing = if app.sheet_state.edit.editing {
        format!(" editing: {} ", app.sheet_state.edit.buffer)
    } else {
        String::new()
    };
    let info_text = format!(" Cell {col_letter}{} {editing}", pos.row + 1);
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
    rat_editor::render_editor(
        frame,
        &app.editor,
        inner,
        "> ",
        Color::Rgb(100, 149, 237),
        "Editor",
    );
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

    // Info panel
    let info = app.rat_tree_state.info();
    let cursor_label = info
        .cursor_node_id
        .map(|id| app.rat_tree_data.node_label(id).to_string())
        .unwrap_or_else(|| "(none)".into());
    let cursor_depth = info
        .cursor_depth
        .map(|d: usize| d.to_string())
        .unwrap_or_else(|| "-".into());
    let is_leaf = info
        .cursor_is_leaf
        .map(|b| if b { "yes" } else { "no" })
        .unwrap_or("-");

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
            Span::styled(
                "Visible rows:  ",
                Style::default().fg(Color::Rgb(140, 140, 140)),
            ),
            Span::styled(
                format!("{}", info.visible_count),
                Style::default().fg(Color::Rgb(100, 149, 237)),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "Cursor node:   ",
                Style::default().fg(Color::Rgb(140, 140, 140)),
            ),
            Span::styled(cursor_label, Style::default().fg(Color::Rgb(144, 238, 144))),
        ]),
        Line::from(vec![
            Span::styled(
                "Cursor depth:  ",
                Style::default().fg(Color::Rgb(140, 140, 140)),
            ),
            Span::styled(cursor_depth, Style::default().fg(Color::Rgb(255, 193, 7))),
        ]),
        Line::from(vec![
            Span::styled(
                "Is leaf:       ",
                Style::default().fg(Color::Rgb(140, 140, 140)),
            ),
            Span::styled(
                is_leaf.to_string(),
                Style::default().fg(Color::Rgb(220, 160, 255)),
            ),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "Keybindings",
            Style::default()
                .fg(Color::Rgb(100, 149, 237))
                .add_modifier(Modifier::BOLD),
        )),
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
            Span::styled(
                "Go to parent",
                Style::default().fg(Color::Rgb(160, 160, 160)),
            ),
        ]),
        Line::from(vec![
            Span::styled("o    ", Style::default().fg(Color::Rgb(100, 149, 237))),
            Span::styled(
                "First child",
                Style::default().fg(Color::Rgb(160, 160, 160)),
            ),
        ]),
        Line::from(vec![
            Span::styled("J/K  ", Style::default().fg(Color::Rgb(100, 149, 237))),
            Span::styled(
                "Next/prev sibling",
                Style::default().fg(Color::Rgb(160, 160, 160)),
            ),
        ]),
        Line::from(vec![
            Span::styled("g/G  ", Style::default().fg(Color::Rgb(100, 149, 237))),
            Span::styled(
                "First/last row",
                Style::default().fg(Color::Rgb(160, 160, 160)),
            ),
        ]),
    ];
    frame.render_widget(Paragraph::new(info_lines), info_inner);
}

fn draw_outline_tab(frame: &mut Frame, area: Rect, app: &mut App) {
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(65), Constraint::Percentage(35)])
        .split(area);

    let outline_block = Block::default()
        .title(Span::styled(
            " Outline Editor ",
            Style::default()
                .fg(Color::Rgb(100, 149, 237))
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(border_style(true));

    let outline_widget = Outline::new(OutlineStyle::default()).block(outline_block);
    app.widget_areas.push(cols[0]);
    frame.render_stateful_widget(outline_widget, cols[0], &mut app.outline_state);

    // Help panel
    let help_block = Block::default()
        .title(Span::styled(
            " Keybindings ",
            Style::default()
                .fg(Color::Rgb(100, 149, 237))
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(60, 60, 60)));
    let help_inner = help_block.inner(cols[1]);
    frame.render_widget(help_block, cols[1]);

    let help = vec![
        Line::from(vec![
            Span::styled("F5 ", Style::default().fg(Color::Rgb(100, 149, 237))),
            Span::styled("Cycle fold", Style::default().fg(Color::Rgb(160, 160, 160))),
        ]),
        Line::from(vec![
            Span::styled("F6 ", Style::default().fg(Color::Rgb(100, 149, 237))),
            Span::styled("Cycle TODO", Style::default().fg(Color::Rgb(160, 160, 160))),
        ]),
        Line::from(vec![
            Span::styled("F7 ", Style::default().fg(Color::Rgb(100, 149, 237))),
            Span::styled("Promote", Style::default().fg(Color::Rgb(160, 160, 160))),
        ]),
        Line::from(vec![
            Span::styled("F8 ", Style::default().fg(Color::Rgb(100, 149, 237))),
            Span::styled("Demote", Style::default().fg(Color::Rgb(160, 160, 160))),
        ]),
        Line::from(vec![
            Span::styled("F9 ", Style::default().fg(Color::Rgb(100, 149, 237))),
            Span::styled("Fold all", Style::default().fg(Color::Rgb(160, 160, 160))),
        ]),
        Line::from(vec![
            Span::styled("F10", Style::default().fg(Color::Rgb(100, 149, 237))),
            Span::styled(
                " Unfold all",
                Style::default().fg(Color::Rgb(160, 160, 160)),
            ),
        ]),
        Line::from(vec![
            Span::styled("F11", Style::default().fg(Color::Rgb(100, 149, 237))),
            Span::styled(
                " Move subtree ↑",
                Style::default().fg(Color::Rgb(160, 160, 160)),
            ),
        ]),
        Line::from(vec![
            Span::styled("F12", Style::default().fg(Color::Rgb(100, 149, 237))),
            Span::styled(
                " Move subtree ↓",
                Style::default().fg(Color::Rgb(160, 160, 160)),
            ),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "Headings",
            Style::default()
                .fg(Color::Rgb(100, 149, 237))
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            format!("  {} headings detected", app.outline_state.headings.len()),
            Style::default().fg(Color::Rgb(140, 140, 140)),
        )),
        Line::from(Span::styled(
            format!("  {} total lines", app.outline_state.line_count()),
            Style::default().fg(Color::Rgb(140, 140, 140)),
        )),
    ];
    frame.render_widget(Paragraph::new(help), help_inner);
}

fn draw_agenda_tab(frame: &mut Frame, area: Rect, app: &mut App) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Min(5)])
        .split(area);

    // View mode hint
    let mode_str = match app.agenda_state.view_mode {
        ViewMode::Day => "Day",
        ViewMode::Week => "Week",
        ViewMode::Month => "Month",
    };
    let hint = Paragraph::new(Line::from(vec![
        Span::styled(" View: ", Style::default().fg(Color::Rgb(140, 140, 140))),
        Span::styled(
            mode_str,
            Style::default()
                .fg(Color::Rgb(100, 149, 237))
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("  d", Style::default().fg(Color::Rgb(100, 149, 237))),
        Span::styled("/day  ", Style::default().fg(Color::Rgb(100, 100, 100))),
        Span::styled("w", Style::default().fg(Color::Rgb(100, 149, 237))),
        Span::styled("/week  ", Style::default().fg(Color::Rgb(100, 100, 100))),
        Span::styled("m", Style::default().fg(Color::Rgb(100, 149, 237))),
        Span::styled("/month  ", Style::default().fg(Color::Rgb(100, 100, 100))),
        Span::styled("←→", Style::default().fg(Color::Rgb(100, 149, 237))),
        Span::styled(" day  ", Style::default().fg(Color::Rgb(100, 100, 100))),
        Span::styled("[]", Style::default().fg(Color::Rgb(100, 149, 237))),
        Span::styled(" week  ", Style::default().fg(Color::Rgb(100, 100, 100))),
        Span::styled("{}", Style::default().fg(Color::Rgb(100, 149, 237))),
        Span::styled(" month", Style::default().fg(Color::Rgb(100, 100, 100))),
    ]));
    frame.render_widget(hint, rows[0]);

    let block = Block::default()
        .title(Span::styled(
            " Agenda ",
            Style::default()
                .fg(Color::Rgb(100, 149, 237))
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(border_style(true));

    let widget = Agenda::new(AgendaStyle::default()).block(block);
    app.widget_areas.push(rows[1]);
    frame.render_stateful_widget(widget, rows[1], &mut app.agenda_state);
}

fn draw_datepicker_tab(frame: &mut Frame, area: Rect, app: &mut App) {
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(24), Constraint::Length(20)])
        .split(area);

    // Calendar
    let cal_block = Block::default()
        .title(Span::styled(
            " Calendar (arrows, [/] month) ",
            Style::default()
                .fg(Color::Rgb(100, 149, 237))
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(border_style(app.focus == 0));

    let cal_widget = CalendarGrid::new(CalendarStyle::default()).block(cal_block);
    app.widget_areas.push(cols[0]);
    frame.render_stateful_widget(cal_widget, cols[0], &mut app.calendar_state);

    // Time + info
    let right = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(4)])
        .split(cols[1]);

    let time_block = Block::default()
        .title(Span::styled(
            " Time (↑↓ digits) ",
            Style::default()
                .fg(Color::Rgb(100, 149, 237))
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(border_style(app.focus == 1));

    let time_widget = TimeInput::new().block(time_block);
    app.widget_areas.push(right[0]);
    frame.render_stateful_widget(time_widget, right[0], &mut app.time_state);

    // Info
    let info_block = Block::default()
        .title(Span::styled(
            " Selection ",
            Style::default()
                .fg(Color::Rgb(100, 149, 237))
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(60, 60, 60)));
    let info_inner = info_block.inner(right[1]);
    frame.render_widget(info_block, right[1]);

    let sel = &app.calendar_state.selected;
    let info = Paragraph::new(vec![
        Line::from(vec![
            Span::styled("Date: ", Style::default().fg(Color::Rgb(140, 140, 140))),
            Span::styled(
                format!("{:04}-{:02}-{:02}", sel.year, sel.month, sel.day),
                Style::default().fg(Color::Rgb(100, 149, 237)),
            ),
        ]),
        Line::from(vec![
            Span::styled("Time: ", Style::default().fg(Color::Rgb(140, 140, 140))),
            Span::styled(
                app.time_state.to_string(),
                Style::default().fg(Color::Rgb(144, 238, 144)),
            ),
        ]),
    ]);
    frame.render_widget(info, info_inner);
}

fn draw_misc_tab(frame: &mut Frame, area: Rect, app: &App) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(16), Constraint::Min(4)])
        .split(area);

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
            Span::styled(
                "rat-widgets     ",
                Style::default()
                    .fg(Color::Rgb(100, 149, 237))
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "Dialogs, spinner, notifications, tree view, scroll, sliders, tabs",
                Style::default().fg(Color::Rgb(160, 160, 160)),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "rat-chrome      ",
                Style::default()
                    .fg(Color::Rgb(180, 220, 255))
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "Overlay placement, backdrop dimming, border/title chrome",
                Style::default().fg(Color::Rgb(160, 160, 160)),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "rat-editor      ",
                Style::default()
                    .fg(Color::Rgb(144, 238, 144))
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "Multi-line text editor with history",
                Style::default().fg(Color::Rgb(160, 160, 160)),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "rat-table       ",
                Style::default()
                    .fg(Color::Rgb(255, 193, 7))
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "Scrollable data table with auto column sizing",
                Style::default().fg(Color::Rgb(160, 160, 160)),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "rat-spreadsheet ",
                Style::default()
                    .fg(Color::Rgb(255, 200, 160))
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "Editable spreadsheet with formulas and cell navigation",
                Style::default().fg(Color::Rgb(160, 160, 160)),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "rat-markdown    ",
                Style::default()
                    .fg(Color::Rgb(220, 160, 255))
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "Markdown to ratatui Spans with syntax highlighting",
                Style::default().fg(Color::Rgb(160, 160, 160)),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "rat-diff        ",
                Style::default()
                    .fg(Color::Rgb(255, 127, 80))
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "In-process unified diff viewer",
                Style::default().fg(Color::Rgb(160, 160, 160)),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "rat-streaming   ",
                Style::default()
                    .fg(Color::Rgb(0, 200, 180))
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "Streaming output buffer with head/tail truncation",
                Style::default().fg(Color::Rgb(160, 160, 160)),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "rat-canvas      ",
                Style::default()
                    .fg(Color::Rgb(255, 100, 100))
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "Infinite canvas viewport with pan/zoom",
                Style::default().fg(Color::Rgb(160, 160, 160)),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "rat-nodegraph   ",
                Style::default()
                    .fg(Color::Rgb(112, 128, 144))
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "Node-based graph editor with typed ports and auto-layout",
                Style::default().fg(Color::Rgb(160, 160, 160)),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "rat-tree        ",
                Style::default()
                    .fg(Color::Rgb(80, 200, 120))
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "Interactive tree navigation with keymap integration",
                Style::default().fg(Color::Rgb(160, 160, 160)),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "rat-outline     ",
                Style::default()
                    .fg(Color::Rgb(255, 160, 100))
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "Folding structured editor with heading hierarchy",
                Style::default().fg(Color::Rgb(160, 160, 160)),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "rat-agenda      ",
                Style::default()
                    .fg(Color::Rgb(200, 160, 255))
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "Day/week/month agenda views with filters",
                Style::default().fg(Color::Rgb(160, 160, 160)),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "rat-datepicker  ",
                Style::default()
                    .fg(Color::Rgb(100, 200, 255))
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "Calendar grid, time input, repeater input",
                Style::default().fg(Color::Rgb(160, 160, 160)),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "+ 12 more       ",
                Style::default()
                    .fg(Color::Rgb(80, 80, 80))
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "fuzzy, capture, backlinks, tags, keymap, leaderkey, etc.",
                Style::default().fg(Color::Rgb(100, 100, 100)),
            ),
        ]),
    ];
    frame.render_widget(Paragraph::new(crates_text), crate_inner);

    let lower = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(65), Constraint::Min(28)])
        .split(rows[1]);

    let theme_block = Block::default()
        .title(Span::styled(
            " WidgetTheme Colors ",
            Style::default()
                .fg(Color::Rgb(100, 149, 237))
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(60, 60, 60)));
    let theme_inner = theme_block.inner(lower[0]);
    frame.render_widget(theme_block, lower[0]);

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

    let preview_block = Block::default()
        .title(Span::styled(
            " Overlay Preview ",
            Style::default()
                .fg(Color::Rgb(180, 220, 255))
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(60, 60, 60)));
    let preview_inner = preview_block.inner(lower[1]);
    frame.render_widget(preview_block, lower[1]);

    if preview_inner.width == 0 || preview_inner.height == 0 {
        return;
    }

    let preview_background = vec![
        Line::from(vec![
            Span::styled("workspace/", Style::default().fg(Color::Rgb(110, 110, 110))),
            Span::styled(
                "src/main.rs",
                Style::default().fg(Color::Rgb(170, 170, 170)),
            ),
        ]),
        Line::from(vec![
            Span::styled("• ", Style::default().fg(Color::Rgb(144, 238, 144))),
            Span::styled(
                "Search ready",
                Style::default().fg(Color::Rgb(170, 170, 170)),
            ),
        ]),
        Line::from(vec![
            Span::styled("• ", Style::default().fg(Color::Rgb(255, 193, 7))),
            Span::styled(
                "3 draft actions",
                Style::default().fg(Color::Rgb(170, 170, 170)),
            ),
        ]),
        Line::from(vec![
            Span::styled("• ", Style::default().fg(Color::Rgb(220, 53, 69))),
            Span::styled(
                "1 conflict to resolve",
                Style::default().fg(Color::Rgb(170, 170, 170)),
            ),
        ]),
    ];
    frame.render_widget(
        Paragraph::new(preview_background).wrap(Wrap { trim: false }),
        preview_inner,
    );

    let model = OverlayModel::default()
        .with_anchor(OverlayAnchor::Center)
        .with_width(OverlaySize::Percent(88))
        .with_height(OverlaySize::Fixed(7))
        .with_title(" Overlay ")
        .with_backdrop(true)
        .with_clear(true);
    let style = OverlayStyle::default()
        .with_border(Style::default().fg(Color::Rgb(180, 220, 255)))
        .with_title(
            Style::default()
                .fg(Color::Rgb(180, 220, 255))
                .add_modifier(Modifier::BOLD),
        )
        .with_backdrop(Style::default().bg(Color::Rgb(14, 17, 24)))
        .with_fill(
            Style::default()
                .fg(Color::Rgb(220, 220, 220))
                .bg(Color::Rgb(28, 32, 44)),
        );
    let layout = overlay_frame(frame, preview_inner, &model, &style);

    if layout.inner.width == 0 || layout.inner.height == 0 {
        return;
    }

    let overlay_body = Paragraph::new(vec![
        Line::from(Span::styled(
            "Centered layout + returned inner rect.",
            Style::default().fg(Color::Rgb(220, 220, 220)),
        )),
        Line::from(vec![
            Span::styled("anchor:", Style::default().fg(Color::Rgb(180, 220, 255))),
            Span::styled(" center  ", Style::default().fg(Color::Rgb(220, 220, 220))),
            Span::styled("size:", Style::default().fg(Color::Rgb(180, 220, 255))),
            Span::styled(" 88% × 7", Style::default().fg(Color::Rgb(220, 220, 220))),
        ]),
    ])
    .wrap(Wrap { trim: true });
    frame.render_widget(overlay_body, layout.inner);
}

// ── Tiler tab ────────────────────────────────────────────────────────────────

fn tiler_column_fixed_width(strip: &Strip, col: usize) -> u16 {
    match strip.columns().get(col).map(|c| c.width_constraint) {
        Some(SizeConstraint::Fixed(w)) => w,
        _ => 30,
    }
}

fn init_tiler(app: &mut App) {
    let panels: &[(&str, Color, usize, SizeConstraint)] = &[
        (
            "Files",
            Color::Rgb(100, 149, 237),
            0,
            SizeConstraint::default(),
        ),
        (
            "Editor",
            Color::Rgb(144, 238, 144),
            1,
            SizeConstraint::Proportion(2.0),
        ),
        (
            "Terminal",
            Color::Rgb(255, 193, 7),
            1,
            SizeConstraint::Proportion(1.0),
        ),
        (
            "Preview",
            Color::Rgb(220, 160, 255),
            2,
            SizeConstraint::default(),
        ),
        (
            "Git",
            Color::Rgb(255, 127, 80),
            3,
            SizeConstraint::default(),
        ),
        (
            "Tests",
            Color::Rgb(0, 200, 180),
            4,
            SizeConstraint::default(),
        ),
        (
            "Logs",
            Color::Rgb(255, 100, 100),
            5,
            SizeConstraint::default(),
        ),
    ];

    for &(name, color, col, hc) in panels {
        let id = app
            .tiler_strip
            .insert_window(col, usize::MAX, SizeConstraint::default(), hc);
        app.tiler_windows.push((id, name.into(), color));
        app.tiler_next_panel += 1;
    }
    // Fixed-width columns so the strip extends beyond the viewport and scrolls.
    app.tiler_strip.resize_column(0, SizeConstraint::Fixed(18));
    app.tiler_strip.resize_column(1, SizeConstraint::Fixed(40));
    app.tiler_strip.resize_column(2, SizeConstraint::Fixed(30));
    app.tiler_strip.resize_column(3, SizeConstraint::Fixed(28));
    app.tiler_strip.resize_column(4, SizeConstraint::Fixed(28));
    app.tiler_strip.resize_column(5, SizeConstraint::Fixed(24));

    if let Some((id, _, _)) = app.tiler_windows.iter().find(|(_, n, _)| n == "Editor") {
        app.tiler_strip.focus_set(*id);
    }
}

fn draw_tiler_tab(frame: &mut Frame, area: Rect, app: &mut App) {
    if area.height < 5 || area.width < 20 {
        return;
    }

    // Reserve 3 rows for header + keybinds, 1 row for scroll bar.
    let header_h: u16 = 3;
    let scrollbar_h: u16 = 1;
    let panel_h = area.height.saturating_sub(header_h + scrollbar_h);

    // Header.
    let focused_name = app
        .tiler_strip
        .focused()
        .and_then(|fid| app.tiler_windows.iter().find(|(id, _, _)| *id == fid))
        .map(|(_, n, _)| n.as_str())
        .unwrap_or("none");
    let col_count = app.tiler_strip.column_count();
    let win_count = app.tiler_windows.len();

    let header = Paragraph::new(vec![
        Line::from(vec![
            Span::styled(
                " Scroll Tiler ",
                Style::default()
                    .fg(Color::Rgb(100, 149, 237))
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("  {win_count} panels  {col_count} columns  focused: {focused_name}"),
                Style::default().fg(Color::Rgb(140, 140, 140)),
            ),
        ]),
        Line::from(vec![
            Span::styled(" ←→", Style::default().fg(Color::Rgb(100, 149, 237))),
            Span::styled(" move  ", Style::default().fg(Color::Rgb(100, 100, 100))),
            Span::styled("↑↓", Style::default().fg(Color::Rgb(100, 149, 237))),
            Span::styled(" stack  ", Style::default().fg(Color::Rgb(100, 100, 100))),
            Span::styled("a", Style::default().fg(Color::Rgb(100, 149, 237))),
            Span::styled(" add  ", Style::default().fg(Color::Rgb(100, 100, 100))),
            Span::styled("s", Style::default().fg(Color::Rgb(100, 149, 237))),
            Span::styled(" split  ", Style::default().fg(Color::Rgb(100, 100, 100))),
            Span::styled("x", Style::default().fg(Color::Rgb(100, 149, 237))),
            Span::styled(" close  ", Style::default().fg(Color::Rgb(100, 100, 100))),
            Span::styled("[]", Style::default().fg(Color::Rgb(100, 149, 237))),
            Span::styled(" resize  ", Style::default().fg(Color::Rgb(100, 100, 100))),
            Span::styled("Home/End", Style::default().fg(Color::Rgb(100, 149, 237))),
            Span::styled(" jump", Style::default().fg(Color::Rgb(100, 100, 100))),
        ]),
        Line::default(),
    ]);
    frame.render_widget(
        header,
        Rect {
            x: area.x,
            y: area.y,
            width: area.width,
            height: header_h,
        },
    );

    // Panel area.
    let panel_area = Rect {
        x: area.x,
        y: area.y + header_h,
        width: area.width,
        height: panel_h,
    };

    let result = tile_compute_layout(&app.tiler_strip, panel_area.width, panel_area.height);

    // Draw panels.
    for vw in &result.visible {
        let (_, ref name, color) = app
            .tiler_windows
            .iter()
            .find(|(id, _, _)| *id == vw.id)
            .map(|(id, n, c)| (*id, n.clone(), *c))
            .unwrap_or((vw.id, "?".into(), Color::Gray));

        let is_focused = app.tiler_strip.focused() == Some(vw.id);
        let border_color = if is_focused {
            color
        } else {
            Color::Rgb(60, 60, 60)
        };
        let title_mod = if is_focused {
            Modifier::BOLD
        } else {
            Modifier::empty()
        };

        let r = Rect {
            x: panel_area.x + vw.rect.x,
            y: panel_area.y + vw.rect.y,
            width: vw.rect.width,
            height: vw.rect.height,
        };
        if r.width < 2 || r.height < 2 {
            continue;
        }

        let block = Block::default()
            .title(Span::styled(
                format!(" {name} "),
                Style::default()
                    .fg(if is_focused {
                        color
                    } else {
                        Color::Rgb(120, 120, 120)
                    })
                    .add_modifier(title_mod),
            ))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color));
        let inner = block.inner(r);
        frame.render_widget(block, r);

        if inner.width > 0 && inner.height > 0 {
            // Show some content: panel name, position info, and a subtle pattern.
            let strip_rect = result.window_rects.get(&vw.id);
            let mut lines: Vec<Line> = Vec::new();
            if let Some(sr) = strip_rect {
                lines.push(Line::from(Span::styled(
                    format!(" {}×{}", sr.width, sr.height),
                    Style::default().fg(Color::Rgb(100, 100, 100)),
                )));
                if is_focused {
                    lines.push(Line::from(Span::styled(
                        format!(" col {} pos {}", sr.x, sr.y),
                        Style::default().fg(Color::Rgb(80, 80, 80)),
                    )));
                }
            }
            // Fill remaining lines with a colored pattern.
            let used = lines.len() as u16;
            for row in used..inner.height {
                let ch = if row % 2 == 0 { "·" } else { " " };
                lines.push(Line::from(Span::styled(
                    ch.repeat(inner.width as usize),
                    Style::default().fg(Color::Rgb(40, 40, 40)),
                )));
            }
            frame.render_widget(Paragraph::new(lines), inner);
        }
    }

    // Scroll indicator bar at bottom.
    let bar_area = Rect {
        x: area.x,
        y: area.y + header_h + panel_h,
        width: area.width,
        height: scrollbar_h,
    };
    if result.strip_extent > 0 && bar_area.width > 2 {
        let track_w = bar_area.width as usize;
        let extent = result.strip_extent.max(1) as f64;
        let vp_frac = (panel_area.width as f64 / extent).min(1.0);
        let thumb_w = ((track_w as f64 * vp_frac) as usize).max(1).min(track_w);
        let thumb_pos = if result.strip_extent > panel_area.width {
            let max_off = (result.strip_extent - panel_area.width) as f64;
            ((result.scroll_offset as f64 / max_off) * (track_w - thumb_w) as f64) as usize
        } else {
            0
        };

        let mut bar = String::with_capacity(track_w);
        for i in 0..track_w {
            if i >= thumb_pos && i < thumb_pos + thumb_w {
                bar.push('█');
            } else {
                bar.push('░');
            }
        }
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(
                bar,
                Style::default().fg(Color::Rgb(60, 60, 60)),
            ))),
            bar_area,
        );
    }
}

// ── Main ─────────────────────────────────────────────────────────────────────

fn main() -> io::Result<()> {
    enable_raw_mode()?;
    execute!(io::stdout(), EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();
    init_tiler(&mut app);
    let tick_rate = Duration::from_millis(50);

    while app.running {
        terminal.draw(|f| draw(f, &mut app))?;

        if event::poll(tick_rate)? {
            match event::read()? {
                Event::Key(key) if key.kind == event::KeyEventKind::Press => {
                    app.handle_key(key.code, key.modifiers);
                }
                Event::Mouse(mouse) => match mouse.kind {
                    MouseEventKind::Down(MouseButton::Left) => {
                        app.handle_mouse_click(mouse.column, mouse.row);
                    }
                    MouseEventKind::ScrollDown => app.handle_scroll(true),
                    MouseEventKind::ScrollUp => app.handle_scroll(false),
                    _ => {}
                },
                _ => {}
            }
        }

        app.tick();
    }

    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture)?;

    Ok(())
}
