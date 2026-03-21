## ADDED Requirements

### Requirement: Intent-only input handlers
`NodeGraphState::handle_mouse_click` and `NodeGraphState::handle_key` SHALL return `Vec<GraphAction>` without directly mutating the contained `Graph`. A separate `apply_action(&mut self, action: &GraphAction)` method SHALL perform the actual graph mutations (add_edge, remove_edge, move node).

#### Scenario: Click returns intent without mutating graph
- **WHEN** user clicks an output port to start wiring, then clicks a compatible input port
- **THEN** `handle_mouse_click` returns `GraphAction::EdgeCreated { source, target }` but the graph's edge count has NOT changed until `apply_action` is called

#### Scenario: Apply action performs mutation
- **WHEN** `apply_action(&GraphAction::EdgeCreated { source, target })` is called
- **THEN** the graph's edge count increases by one

#### Scenario: Caller can reject actions
- **WHEN** `handle_key` returns `GraphAction::NodeMoved { node, x, y }` and the caller does NOT call `apply_action`
- **THEN** the node's position remains unchanged

### Requirement: handle_mouse_drag returns intents
`handle_mouse_drag` SHALL return `Vec<GraphAction>` containing `NodeMoved` intents without mutating node positions directly. The caller SHALL apply them via `apply_action`.

#### Scenario: Drag returns move intents
- **WHEN** two nodes are selected and dragged by (3, 2)
- **THEN** two `GraphAction::NodeMoved` actions are returned, and node positions are unchanged until applied

### Requirement: Selection and mode state remain internal
Selection state (`selected`, `focused`, `focused_port`, `mode`, `selected_edge`) SHALL continue to be mutated directly by input handlers — these are view-layer concerns, not graph model concerns. Only graph-model mutations (edges, positions) SHALL go through the action intent pattern.

#### Scenario: Selection updates immediately
- **WHEN** user clicks a node
- **THEN** `selected` set is updated immediately within `handle_mouse_click`, no apply step needed
