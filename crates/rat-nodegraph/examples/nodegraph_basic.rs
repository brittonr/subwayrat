//! Minimal example: build a graph, auto-layout, and render one frame to stdout.

use rat_nodegraph::layout::{LayoutConfig, auto_layout};
use rat_nodegraph::model::{Graph, PortSpec};
use rat_nodegraph::view::{NodeGraphState, NodeGraphWidget};

use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use std::io;

fn main() -> io::Result<()> {
    // Build a small dataflow graph.
    let mut graph = Graph::new();

    let http = graph.add_node(
        "HTTP Request",
        &[PortSpec::new("url", "string")],
        &[
            PortSpec::new("body", "string"),
            PortSpec::new("status", "number"),
        ],
    );

    let parse = graph.add_node(
        "JSON Parse",
        &[PortSpec::new("input", "string")],
        &[
            PortSpec::new("data", "object"),
            PortSpec::new("error", "error"),
        ],
    );

    let filter = graph.add_node(
        "Filter",
        &[PortSpec::new("data", "object")],
        &[PortSpec::new("matches", "object")],
    );

    let output = graph.add_node(
        "Output",
        &[
            PortSpec::new("data", "object"),
            PortSpec::new("errors", "error"),
        ],
        &[],
    );

    // Wire them up.
    let http_body = graph.node(http).unwrap().output_ports[0].id;
    let parse_input = graph.node(parse).unwrap().input_ports[0].id;
    graph.add_edge(http_body, parse_input).unwrap();

    let parse_data = graph.node(parse).unwrap().output_ports[0].id;
    let filter_data = graph.node(filter).unwrap().input_ports[0].id;
    graph.add_edge(parse_data, filter_data).unwrap();

    let filter_out = graph.node(filter).unwrap().output_ports[0].id;
    let output_data = graph.node(output).unwrap().input_ports[0].id;
    graph.add_edge(filter_out, output_data).unwrap();

    let parse_err = graph.node(parse).unwrap().output_ports[1].id;
    let output_err = graph.node(output).unwrap().input_ports[1].id;
    graph.add_edge(parse_err, output_err).unwrap();

    // Auto-layout left-to-right.
    auto_layout(&mut graph, &LayoutConfig::default());

    // Render one frame.
    let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;
    let size = terminal.size()?;

    let mut state = NodeGraphState::new(graph, size.width, size.height);

    terminal.draw(|frame| {
        let widget = NodeGraphWidget::default();
        frame.render_stateful_widget(widget, frame.area(), &mut state);
    })?;

    Ok(())
}
