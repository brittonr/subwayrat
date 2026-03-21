## MODIFIED Requirements

### Requirement: Edge creation by port-to-port wiring
Users SHALL create edges by clicking an output port then clicking a compatible input port. A "wiring" visual (line from source port to cursor) SHALL be shown while wiring is in progress. Pressing Escape SHALL cancel wiring. Clicking an incompatible port SHALL cancel wiring and show no edge. Input handlers SHALL return `GraphAction::EdgeCreated` intents WITHOUT directly calling `graph.add_edge()`. The caller SHALL apply the action via `apply_action()` to perform the mutation.

#### Scenario: Wire two compatible ports
- **WHEN** user clicks output port "result" (type "string") then clicks input port "data" (type "string")
- **THEN** `handle_mouse_click` returns `GraphAction::EdgeCreated { source, target }` and the edge is NOT added until `apply_action` is called

#### Scenario: Cancel wiring with Escape
- **WHEN** user clicks an output port then presses Escape
- **THEN** wiring is cancelled, no edge is created

#### Scenario: Incompatible port rejects wire
- **WHEN** user clicks output port (type "number") then clicks input port (type "string") with default compatibility
- **THEN** wiring is cancelled, no edge is created

### Requirement: Edge deletion
Users SHALL delete an edge by selecting it and pressing Delete/Backspace, or via a context action. Input handlers SHALL return `GraphAction::EdgeDeleted` intents WITHOUT directly calling `graph.remove_edge()`. The caller SHALL apply the action via `apply_action()`.

#### Scenario: Delete selected edge
- **WHEN** an edge is selected and user presses Delete
- **THEN** `handle_key` returns `GraphAction::EdgeDeleted { source, target }` and the edge is NOT removed until `apply_action` is called

### Requirement: Node dragging
Selected nodes SHALL be movable by mouse drag or arrow keys. Input handlers SHALL return `GraphAction::NodeMoved` intents WITHOUT directly mutating `node.x`/`node.y`. The caller SHALL apply the action via `apply_action()`. When multiple nodes are selected, all move together preserving relative positions.

#### Scenario: Arrow key nudge
- **WHEN** user presses Right arrow with a node selected
- **THEN** `handle_key` returns `GraphAction::NodeMoved { node, x, y }` and the node position is unchanged until `apply_action` is called

#### Scenario: Drag single node
- **WHEN** user drags a selected node from (10,5) by delta (3,2)
- **THEN** `handle_mouse_drag` returns `GraphAction::NodeMoved` with new position (13,7) and the node position is unchanged until `apply_action` is called

### Requirement: Keyboard-only navigation
All interactions SHALL be achievable without a mouse. Tab SHALL cycle focus between nodes. Arrow keys SHALL move the focused/selected node (via action intents). Enter on a port SHALL start wiring mode. Tab in wiring mode SHALL cycle through compatible target ports. Enter SHALL confirm the wire (via action intent).

#### Scenario: Tab cycles node focus
- **WHEN** user presses Tab with node A focused
- **THEN** focus moves to the next node in tab order (immediate state update, no action intent needed)

#### Scenario: Keyboard wiring flow
- **WHEN** user focuses a node, presses Enter on an output port, presses Tab to reach a compatible input port, and presses Enter
- **THEN** `GraphAction::EdgeCreated` is returned and the edge exists after `apply_action`
