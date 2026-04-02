## ADDED Requirements

### Requirement: Builder constructs view trees
`rat-inline` SHALL provide an `InlineView` builder that produces a `ViewTree` (from `ratcore::inline`). The builder SHALL accept any type implementing `InlineWidget`.

#### Scenario: Single widget
- **WHEN** `InlineView::new().push(text_widget)` is called
- **THEN** the result is a `ViewTree` containing one node with the widget's `TypeId`

#### Scenario: Multiple widgets
- **WHEN** `.push(a).push(b)` is chained
- **THEN** the tree contains two nodes in order

### Requirement: Keyed nodes
The builder SHALL support `.keyed(key, widget)` for assigning a string key to a node. The key SHALL be stored on the `ViewNode` for reconciliation.

#### Scenario: Keyed node
- **WHEN** `.keyed("msg-1", markdown_widget)` is called
- **THEN** the resulting node carries key `"msg-1"`

### Requirement: Conditional nodes
The builder SHALL support `.when(condition, |builder| ...)` for conditionally adding nodes. The closure SHALL only execute when the condition is true.

#### Scenario: Condition true
- **WHEN** `.when(true, |b| b.push(spinner))` is called
- **THEN** the tree contains the spinner node

#### Scenario: Condition false
- **WHEN** `.when(false, |b| b.push(spinner))` is called
- **THEN** the tree does not contain the spinner node

### Requirement: Loop nodes
The builder SHALL support `.each(iter, |builder, item| ...)` for generating nodes from an iterator.

#### Scenario: Loop over messages
- **WHEN** `.each(&messages, |b, (i, msg)| b.keyed(format!("m-{i}"), mk_widget(msg)))` is called
- **THEN** the tree contains one node per message with the correct keys

### Requirement: Text shorthand
The builder SHALL support `.text(string)` as shorthand for pushing a basic styled text widget.

#### Scenario: Text shorthand
- **WHEN** `.text("hello")` is called
- **THEN** the tree contains a text node rendering "hello"

### Requirement: InlineWidget trait
`rat-inline` SHALL define an `InlineWidget` trait with `height(&self, width: u16) -> u16` and `render(&self, area: Rect, buf: &mut Buffer)`. Types implementing this trait can participate as leaf nodes in inline view trees.

#### Scenario: Custom widget
- **WHEN** a struct implements `InlineWidget` with `height` returning 3 and `render` writing styled text
- **THEN** the inline renderer allocates 3 rows for it and calls `render` with the correct area
