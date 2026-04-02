## Context

Three repos share work through ratcore:

```
ratcore (pure logic, no UI deps)
├── caldate  → rat-datepicker (ratatui) + met-datepicker (dioxus)
├── fuzzy    → rat-fuzzy (ratatui) + met-command-palette (dioxus)
├── leaderkey→ rat-leaderkey (ratatui) + met-leaderkey (dioxus)
└── tree     → rat-tree (ratatui) + met-tree (dioxus)
```

The inline renderer follows the same split: ratcore owns the reconciler and abstract view tree, subwayrat owns the ratatui terminal backend, meteorite can add a dioxus backend later.

eye-declare demonstrates the target UX — inline scrollback rendering with frame diffing and reconciliation — but it's a monolithic library. We decompose it into the ratcore pattern.

## Goals / Non-Goals

**Goals:**
- `ratcore::inline` module: framework-agnostic view tree nodes, reconciler, commit tracking
- `rat-inline` crate: ratatui backend — renders ratcore view tree into terminal scrollback with ANSI diff output
- Builder API for view tree construction (conditional, loop, keyed nodes)
- Reuse existing rat-* crates as leaf widgets in inline view trees
- DEC synchronized output wrapping to prevent tearing
- Scrollback commit callback for evicting off-screen content
- Pattern consistent with existing ratcore modules (zero UI deps in core, thin UI wrappers)

**Non-Goals:**
- Custom `inline!` proc macro — meteorite already has `rsx!`, and a builder API covers subwayrat. Macro is a follow-up if needed.
- Dioxus integration in this change — meteorite backend comes later as `met-inline`
- Full component lifecycle (hooks, effects, context propagation) — keep the reconciler minimal
- Input handling / focus management — inline mode is output-only for now
- Animated spinners in v1 — needs a tick mechanism, add in follow-up

## Decisions

### 1. Reconciler lives in ratcore, renderer in subwayrat

The reconciler (key/position matching, state preservation, node diffing) is pure logic. It depends on nothing but std. This goes in `ratcore::inline`. The terminal rendering (Buffer allocation, ANSI diff, cursor movement, `\n` growth) goes in `rat-inline` in the subwayrat workspace.

This mirrors the existing pattern: `ratcore::tree` has the data model and visible-row computation, `rat-tree` wraps it in ratatui rendering. `ratcore::leaderkey` has the state machine, `rat-leaderkey` and `met-leaderkey` both wrap it.

**Alternative considered**: Putting everything in `rat-inline`. Rejected because meteorite would need to duplicate the reconciler or depend on a ratatui crate.

### 2. Abstract node types in ratcore, widget trait in rat-inline

ratcore defines:
```rust
// ratcore::inline
pub struct NodeKey(pub String);
pub struct ViewTree { nodes: Vec<ViewNode> }
pub struct ViewNode {
    pub key: Option<NodeKey>,
    pub type_tag: TypeId,
    pub state: Option<Box<dyn Any>>,
}
```

rat-inline defines the widget trait that connects to ratatui:
```rust
// rat-inline
pub trait InlineWidget {
    fn height(&self, width: u16) -> u16;
    fn render(&self, area: Rect, buf: &mut Buffer);
}
```

The reconciler in ratcore works on `ViewNode` (key, type_tag, opaque state). The renderer in rat-inline calls `InlineWidget` methods on the concrete widgets. This keeps ratcore free of ratatui types.

**Alternative considered**: Defining a render trait in ratcore. Rejected — ratcore should not know about `Buffer`, `Rect`, or any rendering types. Different backends have different render signatures.

### 3. Builder API, not a proc macro

View trees are constructed via a builder:
```rust
let view = InlineView::new()
    .text("Processing...")
    .keyed("msg-0", Markdown::new(source.clone()))
    .when(loading, |v| v.push(Spinner::new("Thinking...")))
    .each(&messages, |v, (i, msg)| {
        v.keyed(format!("msg-{i}"), Markdown::new(msg.clone()))
    });
renderer.rebuild(view);
```

Meteorite consumers already have `rsx!`. Subwayrat consumers get a builder that's readable without a proc macro. If the builder proves too verbose, a thin `inline!` proc macro can be added later as sugar — it would just expand to builder calls.

**Alternative considered**: Writing a custom `inline!` proc macro from the start. Rejected — proc macros are expensive to write and maintain, and the builder covers the use cases. The reconciler doesn't care how the tree was constructed.

**Alternative considered**: Using dioxus's `rsx!` macro directly for subwayrat. Rejected — it produces dioxus `VNode` trees, not our `ViewNode` types. Bridging DOM-style mutations to ratatui rendering is more work than the builder.

### 4. Reconciliation algorithm (same as before)

Nodes carry an optional string key. The reconciler matches in two passes:
1. Match keyed nodes by key (stable identity across reorders)
2. Match remaining nodes by position + type (fast path for static layouts)

This lives entirely in `ratcore::inline::reconcile()` — a pure function from `(old_nodes, new_nodes) -> ReconcileResult`. No mutation side effects, no framework coupling.

### 5. Frame diffing stays in rat-inline

The ANSI diff engine (compare Buffer cells, emit escape sequences, DEC sync wrapping, terminal growth via `\n`) is inherently terminal-specific. This stays in `rat-inline` and has no ratcore counterpart. A future `met-inline` would use dioxus's own DOM diffing for its backend.

### 6. Commit tracking in ratcore, terminal height in rat-inline

ratcore tracks which nodes have been "committed" (evicted from the active view because they scrolled out of the visible region). The commit decision is: "this node's rows are entirely above the viewport." ratcore exposes this as a pure function given a viewport height and node heights.

rat-inline provides the actual viewport height from `crossterm::terminal::size()` and fires the commit callback.

## Risks / Trade-offs

- **[ratcore grows another module]** ratcore is 1371 lines today across 4 modules. The inline reconciler adds ~300-400 lines. This is proportional and consistent with the existing modules. → Acceptable.
- **[Two view tree representations]** ratcore has `ViewTree`/`ViewNode`, dioxus has `VNode`/`VirtualDom`. When meteorite adds `met-inline`, it'll need to map between them. → Mitigation: ratcore's types are simple (key + type_tag + opaque state). Mapping from dioxus is straightforward since dioxus already does its own reconciliation — `met-inline` can just use dioxus reconciliation directly and skip ratcore's reconciler, using only the commit tracking.
- **[Builder verbosity]** The builder API is more verbose than `rsx!` or `element!`. → Mitigation: acceptable for the inline use case (typically 5-20 nodes per view). Add proc macro later if real usage shows pain.
- **[Terminal compatibility]** DEC synchronized output not universal. → Mitigation: graceful degradation — flicker without `?2026` but otherwise correct.
