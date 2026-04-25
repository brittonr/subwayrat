# tree-navigation Specification

## Purpose
TBD - created by archiving change rat-tree. Update Purpose after archive.
## Requirements
### Requirement: Cursor moves up and down through visible rows
`TreeState` SHALL support moving the cursor to the previous or next visible row.

#### Scenario: Move down
- **WHEN** the cursor is on row N and row N+1 exists
- **THEN** the cursor SHALL move to row N+1.

#### Scenario: Move down at last row
- **WHEN** the cursor is on the last visible row
- **THEN** the cursor SHALL remain on the last row (clamp, no wrap).

#### Scenario: Move up
- **WHEN** the cursor is on row N > 0
- **THEN** the cursor SHALL move to row N-1.

#### Scenario: Move up at first row
- **WHEN** the cursor is on row 0
- **THEN** the cursor SHALL remain on row 0.

### Requirement: Jump to first and last
`TreeState` SHALL support jumping the cursor to the first or last visible row.

#### Scenario: Jump to first
- **WHEN** the cursor is anywhere
- **THEN** after jump-to-first the cursor SHALL be on row 0.

#### Scenario: Jump to last
- **WHEN** the tree has N visible rows
- **THEN** after jump-to-last the cursor SHALL be on row N-1.

### Requirement: Expand and collapse
`TreeState` SHALL support expanding, collapsing, and toggling the node under the cursor.

#### Scenario: Expand a collapsed node with children
- **WHEN** the cursor is on a collapsed node that has children
- **THEN** the node SHALL be added to the expanded set and visible rows SHALL be recomputed.

#### Scenario: Expand a leaf node
- **WHEN** the cursor is on a leaf node (no children)
- **THEN** expand SHALL be a no-op.

#### Scenario: Collapse an expanded node
- **WHEN** the cursor is on an expanded node
- **THEN** the node SHALL be removed from the expanded set and its descendants SHALL disappear from visible rows.

#### Scenario: Toggle flips state
- **WHEN** toggle is called on a node with children
- **THEN** if the node is expanded it SHALL collapse, and if collapsed it SHALL expand.

### Requirement: Navigate to parent
`TreeState` SHALL support moving the cursor to the parent of the current node.

#### Scenario: Cursor on a non-root node
- **WHEN** the cursor is on a node whose parent is visible
- **THEN** after navigate-to-parent the cursor SHALL be on the parent's visible row.

#### Scenario: Cursor on a root node
- **WHEN** the cursor is on a root node (no parent)
- **THEN** navigate-to-parent SHALL be a no-op.

### Requirement: Navigate to first child
`TreeState` SHALL support moving the cursor to the first child of the current node.

#### Scenario: Expanded node with children
- **WHEN** the cursor is on an expanded node with children
- **THEN** after navigate-to-first-child the cursor SHALL be on the first child's visible row.

#### Scenario: Collapsed node with children
- **WHEN** the cursor is on a collapsed node with children
- **THEN** navigate-to-first-child SHALL first expand the node, then move the cursor to the first child.

#### Scenario: Leaf node
- **WHEN** the cursor is on a leaf node
- **THEN** navigate-to-first-child SHALL be a no-op.

### Requirement: Navigate between siblings
`TreeState` SHALL support moving the cursor to the next or previous sibling at the same depth.

#### Scenario: Next sibling exists
- **WHEN** the cursor is on a node and the next sibling is visible
- **THEN** after next-sibling the cursor SHALL jump to that sibling's visible row, skipping any expanded descendants.

#### Scenario: No next sibling
- **WHEN** the cursor is on the last sibling
- **THEN** next-sibling SHALL be a no-op.

#### Scenario: Previous sibling exists
- **WHEN** the cursor is on a node and a previous sibling exists
- **THEN** after prev-sibling the cursor SHALL jump to that sibling's visible row.

#### Scenario: No previous sibling
- **WHEN** the cursor is on the first sibling
- **THEN** prev-sibling SHALL be a no-op.

### Requirement: Page up and page down
`TreeState` SHALL support page-based cursor movement using a page size (typically viewport height).

#### Scenario: Page down
- **WHEN** page-down is called with page_size P
- **THEN** the cursor SHALL move forward by P rows, clamped to the last visible row.

#### Scenario: Page up
- **WHEN** page-up is called with page_size P
- **THEN** the cursor SHALL move backward by P rows, clamped to row 0.

### Requirement: Scroll follows cursor
When the cursor moves outside the visible viewport, the scroll offset SHALL adjust to keep the cursor visible.

#### Scenario: Cursor moves below viewport
- **WHEN** the cursor row >= scroll_offset + viewport_height
- **THEN** scroll_offset SHALL increase so the cursor row is the last visible line.

#### Scenario: Cursor moves above viewport
- **WHEN** the cursor row < scroll_offset
- **THEN** scroll_offset SHALL decrease to equal the cursor row.

### Requirement: Cursor adjusts on collapse
When a node is collapsed, if the cursor was on one of its descendants, the cursor SHALL move to the collapsed node.

#### Scenario: Cursor on collapsed descendant
- **WHEN** node A is collapsed and the cursor was on a descendant of A
- **THEN** the cursor SHALL move to node A's visible row.

### Requirement: Apply method accepts TreeAction
`TreeState` SHALL provide an `apply(&mut self, action: TreeAction, data: &impl TreeData, viewport_height: usize)` method that executes any `TreeAction` variant against the current state.

#### Scenario: Dispatch up action
- **WHEN** `apply` is called with `TreeAction::Up`
- **THEN** the cursor SHALL move up by one row.

#### Scenario: Dispatch select action
- **WHEN** `apply` is called with `TreeAction::Select`
- **THEN** the method SHALL return the node id of the cursor's current node (for the consumer to act on).
