//! Interactive node graph editor with mouse + keyboard support.
//!
//! Controls:
//!   Tab         — cycle focus between nodes
//!   Arrow keys  — nudge selected node(s)
//!   Enter       — start wiring from focused node's output
//!   Escape      — cancel wiring
//!   Delete      — delete selected edge
//!   Click node  — select it
//!   Shift+click — multi-select
//!   Click port  — start/complete wiring
//!   q           — quit

use rat_nodegraph::layout::{auto_layout, LayoutConfig};
use rat_nodegraph::model::{Graph, PortSpec};
use rat_nodegraph::view::{NodeGraphState, NodeGraphWidget};

use crossterm::event::{self, Event, KeyCode, KeyModifiers, MouseButton, MouseEventKind};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::{execute, event::EnableMouseCapture, event::DisableMouseCapture};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io;

fn main() -> io::Result<()> {
    // Setup terminal.
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run(&mut terminal);

    // Restore terminal.
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result
}

fn run(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> io::Result<()> {
    let mut graph = build_demo_graph();
    auto_layout(&mut graph, &LayoutConfig::default());

    let size = terminal.size()?;
    let mut state = NodeGraphState::new(graph, size.width, size.height);

    loop {
        terminal.draw(|frame| {
            let widget = NodeGraphWidget::default();
            frame.render_stateful_widget(widget, frame.area(), &mut state);
        })?;

        match event::read()? {
            Event::Key(key) => {
                if key.code == KeyCode::Char('q') {
                    return Ok(());
                }

                let key_name = match key.code {
                    KeyCode::Tab => "Tab",
                    KeyCode::Enter => "Enter",
                    KeyCode::Esc => "Escape",
                    KeyCode::Delete => "Delete",
                    KeyCode::Backspace => "Backspace",
                    KeyCode::Up => "Up",
                    KeyCode::Down => "Down",
                    KeyCode::Left => "Left",
                    KeyCode::Right => "Right",
                    _ => continue,
                };

                let shift = key.modifiers.contains(KeyModifiers::SHIFT);
                let _actions = state.handle_key(key_name, shift);
            }

            Event::Mouse(mouse) => match mouse.kind {
                MouseEventKind::Down(MouseButton::Left) => {
                    let shift = mouse.modifiers.contains(KeyModifiers::SHIFT);
                    let _actions =
                        state.handle_mouse_click(mouse.column, mouse.row, shift);
                }
                MouseEventKind::Drag(MouseButton::Left) => {
                    let _actions =
                        state.handle_mouse_drag(mouse.column, mouse.row, 1, 0);
                }
                MouseEventKind::Up(MouseButton::Left) => {
                    let _actions = state.finish_box_select();
                }
                _ => {}
            },

            Event::Resize(w, h) => {
                state.viewport.resize(w, h);
            }

            _ => {}
        }
    }
}

fn build_demo_graph() -> Graph {
    let mut g = Graph::new();

    let trigger = g.add_node(
        "Cron Trigger",
        &[],
        &[PortSpec::new("tick", "any")],
    );

    let http = g.add_node(
        "HTTP Request",
        &[
            PortSpec::new("trigger", "any"),
            PortSpec::new("url", "string"),
        ],
        &[
            PortSpec::new("body", "string"),
            PortSpec::new("status", "number"),
        ],
    );

    let parse = g.add_node(
        "JSON Parse",
        &[PortSpec::new("input", "string")],
        &[
            PortSpec::new("data", "object"),
            PortSpec::new("error", "error"),
        ],
    );

    let filter = g.add_node(
        "Filter",
        &[PortSpec::new("items", "object")],
        &[
            PortSpec::new("pass", "object"),
            PortSpec::new("fail", "object"),
        ],
    );

    let slack = g.add_node(
        "Slack Notify",
        &[PortSpec::new("message", "object")],
        &[],
    );

    let log = g.add_node(
        "Error Log",
        &[PortSpec::new("error", "error")],
        &[],
    );

    // trigger → http
    let t_out = g.node(trigger).unwrap().output_ports[0].id;
    let h_in = g.node(http).unwrap().input_ports[0].id;

    // Use custom compat to allow "any" → anything.
    g.set_compatibility(Box::new(|src, tgt| src == tgt || src == "any" || tgt == "any"));

    g.add_edge(t_out, h_in).unwrap();

    // http body → parse input
    let h_body = g.node(http).unwrap().output_ports[0].id;
    let p_in = g.node(parse).unwrap().input_ports[0].id;
    g.add_edge(h_body, p_in).unwrap();

    // parse data → filter items
    let p_data = g.node(parse).unwrap().output_ports[0].id;
    let f_in = g.node(filter).unwrap().input_ports[0].id;
    g.add_edge(p_data, f_in).unwrap();

    // filter pass → slack
    let f_pass = g.node(filter).unwrap().output_ports[0].id;
    let s_in = g.node(slack).unwrap().input_ports[0].id;
    g.add_edge(f_pass, s_in).unwrap();

    // parse error → error log
    let p_err = g.node(parse).unwrap().output_ports[1].id;
    let l_in = g.node(log).unwrap().input_ports[0].id;
    g.add_edge(p_err, l_in).unwrap();

    g
}
