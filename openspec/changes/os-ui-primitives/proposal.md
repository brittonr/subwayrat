## Why

The workspace has individual widgets (select lists, tab bars, progress bars, sliders) and layout primitives (strip layout, window management), but lacks the structural UI components that desktop operating systems provide: title bars, menu bars, context menus, toolbars, status bars, split panes, and dialog frames. Anyone building a TUI application that mimics an OS or IDE experience has to hand-roll these from raw ratatui blocks. A dedicated crate for these components fills the gap between low-level widgets and full application layout.

## What Changes

- New crate `rat-chrome` providing OS-style structural UI components for ratatui.
- Title bar widget with text, alignment options, and indicator symbols (close/minimize/maximize or custom).
- Menu bar with dropdown menus, nested submenus, accelerator key display, and keyboard navigation.
- Context menu as a positioned popup with items, separators, submenus, and disabled states.
- Toolbar widget with button items, toggle items, separators, and horizontal/vertical orientation.
- Status bar with left/center/right segmented regions, each accepting arbitrary spans.
- Breadcrumb widget with clickable path segments, truncation from the left when space is tight, and configurable separators.
- Tooltip overlay that renders a floating text box at a given position, clipped to viewport bounds.
- Split pane container with a draggable divider between two child regions, supporting horizontal and vertical splits with configurable ratio or fixed divider position.
- Dialog frame compositing a title bar, content area, and button row into a modal overlay with border.

## Capabilities

### New Capabilities

- `title-bar`: Title bar rendering with label, alignment, and indicator buttons (close/min/max or custom glyphs). Pure data model tracks which indicator is focused.
- `menu-system`: Menu bar + dropdown + submenu hierarchy. Model tracks open menu path, focused item, and accelerator keys. Navigation: arrow keys move between menus and items, Enter activates, Escape closes.
- `context-menu`: Positioned popup menu with items, separators, disabled entries, and nested submenus. Shares item model with menu-system but renders as a floating overlay at (x, y).
- `toolbar`: Horizontal or vertical strip of button/toggle items with separators. Model tracks focused item and toggle states. Supports icon+label or icon-only items.
- `status-bar`: Bottom bar divided into left, center, and right segments. Each segment holds a `Vec<Span>`. No interactive state — purely a rendering primitive.
- `breadcrumb`: Path segment display with configurable separator char. Truncates from the left (dropping leading segments, replacing with ellipsis) when the total width exceeds available space. Model tracks which segment is active.
- `tooltip`: Floating text rendered at a given (x, y), automatically repositioned to stay within viewport bounds. Supports multi-line content and configurable border.
- `split-pane`: Divides a Rect into two child regions with a draggable divider. Supports horizontal (left/right) and vertical (top/bottom) orientation. Model tracks divider position as either a ratio (0.0-1.0) or fixed cell count from one edge. Min-size constraints prevent collapsing either pane below a threshold.
- `dialog-frame`: Composite widget assembling a title bar, bordered content area, and a row of action buttons (Ok, Cancel, custom). Renders as a centered overlay with optional background dimming. Model tracks which button is focused.

### Modified Capabilities

(none)

## Impact

- New crate `crates/rat-chrome` added to workspace `Cargo.toml`.
- Depends on `ratatui` (workspace) and `unicode-width` (workspace).
- No dependency on `rat-keymap` — consumers wire their own key handling to the model methods, same as rat-widgets does. Keymap integration can be added later.
- No breaking changes to existing crates.
- `rat-widgets` remains unchanged; `rat-chrome` targets higher-level structural components that compose around child content rather than standalone leaf widgets.
