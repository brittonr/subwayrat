## ADDED Requirements

### Requirement: Heading detection and index
The system SHALL maintain a `Vec<HeadingInfo>` index parallel to the editor buffer. Each `HeadingInfo` SHALL contain: line number, heading level (1+), fold state (folded/children-visible/all-visible), TODO state (Option<String>), priority (Option<char>), tags (Vec<String>), and title text. The index SHALL be rebuilt whenever the buffer content changes.

#### Scenario: Parse headings from buffer
- **WHEN** the buffer contains lines `["* TODO Foo", "body", "** Bar :tag1:tag2:", "more body"]`
- **THEN** the heading index contains two entries: level 1 at line 0 with TODO="TODO", title="Foo"; level 2 at line 2 with tags=["tag1","tag2"], title="Bar"

#### Scenario: No headings
- **WHEN** the buffer contains only plain text with no heading markers
- **THEN** the heading index is empty

#### Scenario: Index updates on edit
- **WHEN** a new heading line is inserted into the buffer
- **THEN** the heading index reflects the new heading on the next state update

### Requirement: Configurable heading syntax
The system SHALL accept a `HeadingSyntax` configuration that defines how headings are detected. The default SHALL recognize lines starting with one or more `*` followed by a space (org-mode style). An alternative markdown syntax (`#` prefix) SHALL also be provided. Custom syntax SHALL be possible by implementing a `HeadingParser` trait.

#### Scenario: Org-style headings
- **WHEN** `HeadingSyntax::Org` is configured and the line is `** TODO Task`
- **THEN** the parser returns level=2, todo=Some("TODO"), title="Task"

#### Scenario: Markdown-style headings
- **WHEN** `HeadingSyntax::Markdown` is configured and the line is `## Task`
- **THEN** the parser returns level=2, todo=None, title="Task"

### Requirement: Visibility cycling
The system SHALL support three visibility states per heading: **folded** (only the heading line visible), **children** (heading + immediate child headings visible, their bodies hidden), **all** (entire subtree visible). Cycling advances folded→children→all→folded.

#### Scenario: Fold a heading
- **WHEN** a heading at line 5 with 10 lines of body and 2 child headings is set to folded
- **THEN** only line 5 is rendered; lines 6-16 are hidden; a fold indicator is shown

#### Scenario: Cycle to children
- **WHEN** a folded heading is cycled once
- **THEN** the heading line and its direct child heading lines are visible; body text and grandchild subtrees remain hidden

#### Scenario: Cycle to all
- **WHEN** a children-visible heading is cycled once
- **THEN** all lines in the subtree are visible

#### Scenario: Global fold/unfold
- **WHEN** a global fold action is triggered
- **THEN** all headings are set to folded state simultaneously

### Requirement: Structural editing operations
The system SHALL support: **promote** (decrease heading level by 1, including all children), **demote** (increase heading level by 1, including all children), **move subtree up** (swap subtree with previous sibling), **move subtree down** (swap subtree with next sibling). Each operation SHALL modify the buffer content and update the heading index.

#### Scenario: Promote a level-2 heading
- **WHEN** promote is called on a `** Task` heading with child `*** Sub`
- **THEN** the heading becomes `* Task` and the child becomes `** Sub`

#### Scenario: Promote at level 1
- **WHEN** promote is called on a level-1 heading
- **THEN** no change occurs (cannot promote above level 1)

#### Scenario: Move subtree down
- **WHEN** move-down is called on the first of two sibling headings at level 2
- **THEN** the entire first subtree (heading + body + children) moves below the second subtree

#### Scenario: Move subtree down at last sibling
- **WHEN** move-down is called on the last sibling heading at its level
- **THEN** no change occurs

### Requirement: TODO state cycling
The system SHALL accept a configurable list of TODO states (e.g., `["TODO", "IN_PROGRESS", "DONE"]`). A cycle action on a heading SHALL advance its TODO keyword to the next state, wrapping to None after the last state.

#### Scenario: Cycle TODO to next state
- **WHEN** the current heading has TODO state "TODO" and the state list is ["TODO", "DONE"]
- **THEN** after cycling, the heading line is updated to show "DONE" and the heading index reflects the change

#### Scenario: Cycle past last state
- **WHEN** the current heading has TODO state "DONE" (last in list)
- **THEN** after cycling, the TODO keyword is removed from the heading line

#### Scenario: Cycle from no state
- **WHEN** the current heading has no TODO keyword
- **THEN** after cycling, the first keyword in the state list is inserted

### Requirement: Outline rendering
The system SHALL implement `StatefulWidget` for ratatui. Visible lines SHALL be computed by filtering the buffer through fold state. Heading lines SHALL be styled differently from body text (configurable via `OutlineStyle`). Fold indicators SHALL appear in a gutter column. TODO keywords, priority cookies, and tags SHALL each have distinct styles.

#### Scenario: Render folded document
- **WHEN** a document with 3 top-level headings (all folded) is rendered in a 20-line area
- **THEN** exactly 3 lines are rendered, each showing the heading text and a fold indicator

#### Scenario: Gutter shows fold markers
- **WHEN** a heading is folded
- **THEN** a `▶` (or configurable character) appears in the gutter; when expanded it shows `▼`

### Requirement: OutlineState and Action enum
The system SHALL expose `OutlineState` containing the `Editor` buffer, heading index, fold states, TODO keyword list, heading syntax config, cursor position, and scroll offset. An `Action` enum SHALL include: `CycleVisibility`, `CycleVisibilityGlobal`, `Promote`, `Demote`, `MoveSubtreeUp`, `MoveSubtreeDown`, `CycleTodo`, `InsertHeading`, `InsertSubheading`, and all text editing actions delegated to the inner `Editor`. A `handle_action(state, action)` function SHALL process actions.

#### Scenario: Handle CycleVisibility action
- **WHEN** `handle_action(state, Action::CycleVisibility)` is called with cursor on a folded heading
- **THEN** the heading's fold state advances to children-visible

#### Scenario: Handle text editing action
- **WHEN** `handle_action(state, Action::Edit(EditorAction::InsertChar('x')))` is called
- **THEN** the character is inserted at the cursor position and the heading index is updated if the edit changed a heading line
