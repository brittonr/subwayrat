## ADDED Requirements

### Requirement: Single and multi-node selection
The interaction layer SHALL support selecting a single node by clicking it
or pressing Enter when focused. Multi-select SHALL be supported via
Shift+click or box selection. Clicking empty canvas SHALL clear selection.

#### Scenario: Click to select single node
- **WHEN** user clicks on a node
- **THEN** that node becomes the only selected node, and a
  `GraphAction::SelectionChanged` is returned

#### Scenario: Shift-click for multi-select
- **WHEN** user shift-clicks a second node while one is already selected
- **THEN** both nodes are selected

#### Scenario: Click empty space clears selection
- **WHEN** user clicks on empty canvas area
- **THEN** selection is cleared and `GraphAction::SelectionChanged` is
  returned with an empty set

### Requirement: Node dragging
Selected nodes SHALL be movable by mouse drag or arrow keys. Dragging SHALL
update the node's canvas position. When multiple nodes are selected, all
move together preserving relative positions.

#### Scenario: Drag single node
- **WHEN** user drags a selected node from (10,5) by delta (3,2)
- **THEN** the node's position becomes (13,7) and `GraphAction::NodeMoved`
  is returned

#### Scenario: Arrow key nudge
- **WHEN** user presses Right arrow with a node selected
- **THEN** the node moves one grid unit right and `GraphAction::NodeMoved`
  is returned

#### Scenario: Multi-node drag preserves spacing
- **WHEN** nodes at (10,5) and (20,5) are both selected and dragged by (5,0)
- **THEN** nodes end up at (15,5) and (25,5)

### Requirement: Edge creation by port-to-port wiring
Users SHALL create edges by clicking an output port then clicking a
compatible input port. A "wiring" visual (line from source port to cursor)
SHALL be shown while wiring is in progress. Pressing Escape SHALL cancel
wiring. Clicking an incompatible port SHALL cancel wiring and show no edge.

#### Scenario: Wire two compatible ports
- **WHEN** user clicks output port "result" (type "string") then clicks
  input port "data" (type "string")
- **THEN** an edge is created and `GraphAction::EdgeCreated` is returned

#### Scenario: Cancel wiring with Escape
- **WHEN** user clicks an output port then presses Escape
- **THEN** wiring is cancelled, no edge is created

#### Scenario: Incompatible port rejects wire
- **WHEN** user clicks output port (type "number") then clicks input port
  (type "string") with default compatibility
- **THEN** wiring is cancelled, no edge is created

### Requirement: Edge deletion
Users SHALL delete an edge by selecting it and pressing Delete/Backspace,
or via a context action. Edge selection SHALL be possible by clicking on
the edge line or by cycling through edges of a selected node.

#### Scenario: Delete selected edge
- **WHEN** an edge is selected and user presses Delete
- **THEN** the edge is removed and `GraphAction::EdgeDeleted` is returned

### Requirement: Box selection
Users SHALL drag a rectangle on empty canvas to select all nodes whose
bounding boxes intersect the rectangle.

#### Scenario: Box select multiple nodes
- **WHEN** user drags a selection rectangle from (0,0) to (50,30) and
  three nodes fall within that area
- **THEN** all three nodes become selected

### Requirement: Keyboard-only navigation
All interactions SHALL be achievable without a mouse. Tab SHALL cycle
focus between nodes. Arrow keys SHALL move the focused/selected node.
Enter on a port SHALL start wiring mode. Tab in wiring mode SHALL cycle
through compatible target ports. Enter SHALL confirm the wire.

#### Scenario: Tab cycles node focus
- **WHEN** user presses Tab with node A focused
- **THEN** focus moves to the next node in tab order

#### Scenario: Keyboard wiring flow
- **WHEN** user focuses a node, presses Enter on an output port, presses
  Tab to reach a compatible input port, and presses Enter
- **THEN** an edge is created between the two ports
