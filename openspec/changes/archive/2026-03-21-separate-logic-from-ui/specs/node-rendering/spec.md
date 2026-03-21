## MODIFIED Requirements

### Requirement: StatefulWidget implementation
The node graph renderer SHALL implement ratatui's `StatefulWidget` trait, accepting a `&mut NodeGraphState` that holds the graph model, viewport, selection state, and interaction state. The graph SHALL be accessed via `state.graph` during rendering. Input handlers on `NodeGraphState` SHALL return action intents; rendering reads the graph but does not mutate it.

#### Scenario: Render via StatefulWidget
- **WHEN** caller calls `frame.render_stateful_widget(NodeGraphWidget, area, &mut state)`
- **THEN** the graph is rendered into the given area using the viewport and current graph data from `state.graph`

#### Scenario: Render does not mutate graph
- **WHEN** `StatefulWidget::render` is called
- **THEN** no edges are added/removed and no node positions change during rendering (only viewport metadata like `area` and `tab_order` are updated)
