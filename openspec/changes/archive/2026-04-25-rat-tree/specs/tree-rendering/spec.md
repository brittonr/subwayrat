## ADDED Requirements

### Requirement: TreeStyle controls visual appearance
The crate SHALL provide a `TreeStyle` struct with configurable fields for indent width, guide characters, expand/collapse indicators, icon style, selected row style, and normal row style.

#### Scenario: Default style
- **WHEN** `TreeStyle::default()` is called
- **THEN** it SHALL produce a usable style with indent width 2, ASCII guide characters (`│`, `├`, `└`, `─`), expand indicator `▸`, collapse indicator `▾`, and visible highlight on the selected row.

#### Scenario: Custom style via builder
- **WHEN** a consumer sets `with_indent_width(4)` and `with_guide_chars("| ", "|-", "`-", "- ")`
- **THEN** the style SHALL use those values during rendering.

### Requirement: Indented rows with guide lines
Each visible row SHALL be rendered with leading whitespace and guide characters proportional to its depth. Guide lines SHALL connect siblings and show the tree structure.

#### Scenario: Root node at depth 0
- **WHEN** a root node is rendered
- **THEN** no indent or guide characters SHALL precede the label.

#### Scenario: Child at depth 2
- **WHEN** a node at depth 2 is rendered
- **THEN** 2 indent levels of guide characters SHALL precede the label.

#### Scenario: Last child uses corner guide
- **WHEN** a node is the last sibling at its level
- **THEN** the guide character SHALL be the corner connector (`└─`) instead of the tee connector (`├─`).

### Requirement: Expand/collapse indicator
Nodes with children SHALL display an indicator showing whether they are expanded or collapsed. Leaf nodes SHALL display no indicator.

#### Scenario: Collapsed node with children
- **WHEN** a collapsed node with children is rendered
- **THEN** the expand indicator (e.g., `▸`) SHALL appear before the label.

#### Scenario: Expanded node with children
- **WHEN** an expanded node is rendered
- **THEN** the collapse indicator (e.g., `▾`) SHALL appear before the label.

#### Scenario: Leaf node
- **WHEN** a leaf node is rendered
- **THEN** a space (same width as the indicator) SHALL appear instead, keeping labels aligned.

### Requirement: Node icon support
If a node provides an icon via `TreeData::node_icon`, it SHALL be rendered between the indicator and the label using the icon style from `TreeStyle`.

#### Scenario: Node with icon
- **WHEN** `node_icon` returns `Some("📁")`
- **THEN** the icon SHALL appear between the expand indicator and the label text.

#### Scenario: Node without icon
- **WHEN** `node_icon` returns `None`
- **THEN** no extra space SHALL be inserted for the icon.

### Requirement: Selected row highlight
The row at the cursor position SHALL be rendered with the selected style from `TreeStyle`.

#### Scenario: Cursor on row 3
- **WHEN** the cursor is on visible row 3
- **THEN** row 3 SHALL use `TreeStyle.selected_style` and all other rows SHALL use `TreeStyle.normal_style`.

### Requirement: Viewport scrolling
Only rows within `[scroll_offset, scroll_offset + viewport_height)` SHALL be rendered. Rows outside this range SHALL be skipped.

#### Scenario: Scroll offset 10, viewport 20
- **WHEN** scroll_offset is 10 and viewport_height is 20
- **THEN** only visible rows 10 through 29 SHALL be rendered.

### Requirement: TreeInfo snapshot
The crate SHALL provide a `TreeInfo` struct computed from current state, containing: total visible row count, cursor node id, cursor depth, and whether the cursor is on a leaf node.

#### Scenario: Info reflects state
- **WHEN** `TreeState::info()` is called
- **THEN** the returned `TreeInfo` SHALL match the current cursor position and visible row count.

### Requirement: StatefulWidget implementation
The tree widget SHALL implement ratatui's `StatefulWidget` trait with `TreeState` as the state type, accepting `TreeStyle` and `&dyn TreeData` via builder methods.

#### Scenario: Render in a Frame
- **WHEN** `frame.render_stateful_widget(tree_widget, area, &mut state)` is called
- **THEN** the tree SHALL render within the given `Rect` using the provided state.
