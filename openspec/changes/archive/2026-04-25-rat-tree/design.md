## Context

The subwayrat workspace contains ratatui widget crates, each with focused scope. Existing crates provide flat-list navigation (rat-table), text editing (rat-editor), tree algorithms (rat-branches), and a keymap system (rat-keymap). There is no interactive tree widget.

The workspace follows consistent patterns: state struct + style struct + info struct, builder-pattern configuration, `StatefulWidget` rendering, and Rust 2024 edition.

`rat-branches` already has a `TreeNode` trait (id/parent_id) and algorithms for walking to root, finding leaves, and counting children. `rat-keymap` provides a generic modal `Keymap<A, M>` that resolves `KeyEvent` → action.

## Goals / Non-Goals

**Goals:**
- Generic tree widget that works with any data type implementing a node trait.
- Expand/collapse with state persistence across renders.
- Cursor navigation: up, down, into children, back to parent, siblings, first/last.
- Scroll management when the tree is taller than the viewport.
- Configurable styles for indent guides, icons, selection highlight.
- Direct integration with `rat-keymap` so consumers wire key events to `TreeAction` variants.
- Pure navigation functions (no side effects) so state mutations are testable without a terminal.

**Non-Goals:**
- Drag-and-drop or reordering nodes via the widget.
- Inline editing of node labels (use rat-editor separately).
- Multi-selection (single cursor only for v1; can extend later).
- Async/lazy loading of child nodes (consumers pre-load the full tree).
- Mouse interaction (keyboard-first; mouse can be added later).

## Decisions

### Flat visible-row model instead of recursive rendering

The tree state maintains a flat `Vec` of visible rows computed from the tree structure plus expand/collapse state. Each row records its depth, node index, and whether it has children. Navigation and rendering operate on this flat list.

**Why over recursive walk during render:** Flat rows make cursor indexing, scrolling, and hit-testing trivial. The visible-row vec is recomputed only when expand/collapse state changes, not every frame.

### Node trait with children iterator, not parent pointers

`rat-branches::TreeNode` uses parent pointers (`parent_id`). For tree rendering we need top-down traversal (iterate children in order). The widget defines its own `TreeData` trait:

```rust
pub trait TreeData {
    fn root_count(&self) -> usize;
    fn root(&self, index: usize) -> usize; // → node id
    fn child_count(&self, node: usize) -> usize;
    fn child(&self, node: usize, index: usize) -> usize; // → child node id
    fn node_label(&self, node: usize) -> &str;
    fn node_icon(&self, node: usize) -> Option<&str> { None }
}
```

**Why a new trait:** Parent-pointer trees require collecting and sorting children at each level. A top-down trait lets the consumer store children however they want (vec, arena, etc.) and avoids allocation during traversal. Consumers with `TreeNode`-shaped data can write a trivial adapter.

### TreeAction enum with keymap integration

A `TreeAction` enum covers all navigation operations the widget supports. The crate provides `default_keymap()` returning `Keymap<TreeAction, ()>` with vim-style defaults. Consumers can override or provide their own modal keymap.

```rust
pub enum TreeAction {
    Up, Down,
    Expand, Collapse, Toggle,
    Parent, FirstChild,
    NextSibling, PrevSibling,
    First, Last,
    PageUp, PageDown,
    Select,  // signals "user chose this node"
}
```

Consumers call `keymap.resolve(&mode, &event)` to get an `Option<TreeAction>`, then pass it to `TreeState::apply(action)`.

**Why separate resolution from application:** Keeps the widget independent of event handling. Consumers can map the same action from keys, mouse, or programmatic calls. The keymap crate stays a peer dependency, not a hard coupling inside rendering.

### State / Style / Info separation

Following rat-table conventions:

- `TreeState` — mutable: expanded set (`BTreeSet<usize>`), cursor index (into flat visible rows), scroll offset, cached visible rows.
- `TreeStyle` — immutable config: indent width, guide chars, expand/collapse chars, icon style, selected style, normal style.
- `TreeInfo` — computed snapshot: total visible rows, cursor node id, depth, whether cursor is at leaf.

### BTreeSet for expanded nodes

Using `BTreeSet<usize>` instead of `HashSet`. Deterministic iteration order avoids nondeterminism (napkin lesson from earlier).

## Risks / Trade-offs

- **[Large trees]** → Flat visible-row recomputation is O(visible nodes) on expand/collapse. For trees with millions of visible nodes this could lag. Mitigation: incremental recomputation can be added later; most TUI trees are <10k nodes.
- **[Trait impedance]** → Consumers with `rat-branches::TreeNode` data need an adapter. Mitigation: provide a `SimpleTree` adapter struct in the crate that takes a flat `Vec<(id, parent_id, label)>` and builds the top-down lookup.
- **[No mouse]** → Keyboard-only in v1. Mitigation: mouse click-to-select and click-to-toggle can be added without breaking changes since it only adds methods to `TreeState`.
