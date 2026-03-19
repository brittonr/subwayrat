## ADDED Requirements

### Requirement: Nodes render as bordered boxes with port rows
Each node SHALL render as a bordered rectangle. The top border SHALL contain
the node label. Input ports SHALL be listed on the left edge, output ports on
the right edge. Port labels SHALL be visible inside the box. The selected
node SHALL use a distinct border style (e.g., double-line or bold).

#### Scenario: Single node renders with ports
- **WHEN** a node "Transform" with inputs ["data"] and outputs ["result",
  "error"] is rendered at position (0, 0)
- **THEN** the rendered output shows a bordered box with "Transform" in the
  header, "data" on the left with a port marker, and "result"/"error" on the
  right with port markers

#### Scenario: Selected node has distinct border
- **WHEN** a node is marked as selected
- **THEN** its border style differs from unselected nodes (double-line or
  highlighted color)

### Requirement: Edges render as Manhattan-routed lines
Edges between ports SHALL render using Unicode box-drawing characters
(`─`, `│`, `┐`, `└`, `┘`, `┌`) following horizontal-then-vertical or
vertical-then-horizontal Manhattan routing. Edge lines SHALL not pass
through node interiors.

#### Scenario: Horizontal edge between adjacent nodes
- **WHEN** node A output is at column 20 and node B input is at column 30,
  same row
- **THEN** the edge renders as a horizontal line of `─` characters from the
  output port to the input port

#### Scenario: Edge with vertical segment
- **WHEN** node A output is at row 5 and node B input is at row 15
- **THEN** the edge renders with a horizontal segment, a corner character,
  a vertical segment, another corner, and a horizontal segment into the
  target port

### Requirement: Viewport culling for performance
The renderer SHALL skip nodes and edges that fall entirely outside the
visible `rat-canvas::Viewport` area. Only nodes whose bounding box
intersects the viewport, and edges with at least one endpoint in the
viewport, SHALL be drawn.

#### Scenario: Off-screen node not rendered
- **WHEN** viewport shows canvas area (0,0)-(80,24) and a node is positioned
  at (200, 200)
- **THEN** no cells are written for that node

#### Scenario: Partially visible node is rendered
- **WHEN** a node's bounding box spans (75,0)-(90,5) and viewport covers
  (0,0)-(80,24)
- **THEN** the visible portion of the node (columns 75-80) is rendered and
  the rest is clipped

### Requirement: Port type coloring
Each port type_tag SHALL map to a display color. The color SHALL be applied
to the port marker character and the edge line originating from that port.
Callers SHALL be able to provide a custom `Fn(&str) -> Color` mapping.
A default palette SHALL be provided.

#### Scenario: Ports with same type share color
- **WHEN** two ports both have type_tag "string"
- **THEN** both port markers and their edge lines render in the same color

#### Scenario: Custom color mapping
- **WHEN** caller provides a mapping where "number" → Green and "string" → Cyan
- **THEN** ports and edges use those colors

### Requirement: StatefulWidget implementation
The node graph renderer SHALL implement ratatui's `StatefulWidget` trait,
accepting a `&mut NodeGraphState` that holds the graph model, viewport,
selection state, and interaction state.

#### Scenario: Render via StatefulWidget
- **WHEN** caller calls `frame.render_stateful_widget(NodeGraphWidget, area, &mut state)`
- **THEN** the graph is rendered into the given area using the viewport and
  current graph data from state
