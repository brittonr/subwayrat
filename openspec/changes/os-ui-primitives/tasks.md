## 1. Crate Scaffolding

- [ ] 1.1 Create `crates/rat-chrome/Cargo.toml` with deps: ratatui (workspace), unicode-width (workspace); edition 2024, MIT
- [ ] 1.2 Add `crates/rat-chrome` to workspace `Cargo.toml` members
- [ ] 1.3 Create `src/lib.rs` with module declarations and public re-exports
- [ ] 1.4 Create shared `MenuItem` enum and `MenuNav` helper in `src/menu_item.rs`

## 2. Title Bar

- [ ] 2.1 Define `TitleBarModel` struct (label, alignment, indicators vec, focused indicator index)
- [ ] 2.2 Implement indicator focus navigation (move_left, move_right, clear_focus)
- [ ] 2.3 Define `TitleBarStyle` struct with builder methods (background, label, indicator, focused indicator styles)
- [ ] 2.4 Implement title bar render function — label with truncation/alignment, indicators right-aligned with gap
- [ ] 2.5 Tests: label truncation, center alignment with indicators, focus movement, boundary clamping

## 3. Menu System

- [ ] 3.1 Define `MenuBarModel` struct (top-level labels, active index, open state, dropdown items per menu)
- [ ] 3.2 Implement menu bar navigation: move_left, move_right, open, close, switch while open
- [ ] 3.3 Implement `MenuNav` shared navigation: move_up/move_down through items skipping separators and disabled, wrapping at boundaries
- [ ] 3.4 Implement submenu enter/leave tracking (open submenu path as a stack of indices)
- [ ] 3.5 Implement activate: return selected item label, close menu
- [ ] 3.6 Define `MenuBarStyle` and `DropdownStyle` structs with builders
- [ ] 3.7 Implement menu bar render function — horizontal label row with active highlight
- [ ] 3.8 Implement dropdown render function — bordered item list with accelerator hints, separator lines, submenu indicators, disabled dimming
- [ ] 3.9 Implement dropdown width computation (max of content width and minimum)
- [ ] 3.10 Tests: navigation skips disabled/separators, wrapping, submenu open/close, activate returns correct item, width computation

## 4. Context Menu

- [ ] 4.1 Define `ContextMenuModel` struct (items, anchor position, visible flag, menu nav state)
- [ ] 4.2 Implement show/dismiss, navigation (delegates to MenuNav), activate
- [ ] 4.3 Implement context menu render function — positioned floating box with Clear, viewport-boundary repositioning
- [ ] 4.4 Tests: repositioning when overflowing right edge, bottom edge, dismiss clears submenu stack

## 5. Toolbar

- [ ] 5.1 Define `ToolbarItem` enum (Button, Toggle, Separator) with icon/label fields
- [ ] 5.2 Define `ToolbarModel` struct (items, focused index, toggle states, orientation)
- [ ] 5.3 Implement focus navigation (skip separators), activate (return index, toggle state change)
- [ ] 5.4 Define `ToolbarStyle` struct with builder methods
- [ ] 5.5 Implement toolbar render function — items along primary axis with gap, separator rendering, overflow indicator
- [ ] 5.6 Implement overflow scrolling (visible window shifts when focus moves into overflow)
- [ ] 5.7 Tests: focus skips separators, toggle state change, overflow indicator appears, scroll on focus

## 6. Status Bar

- [ ] 6.1 Implement `StatusBar` struct with builder methods for left/center/right `Vec<Span>` and background style
- [ ] 6.2 Implement render function — left-align left segment, center center segment, right-align right segment, fill background
- [ ] 6.3 Implement center truncation logic when left + right exceed available space
- [ ] 6.4 Tests: three-segment layout, center truncation, empty segments, background fill

## 7. Breadcrumb

- [ ] 7.1 Define `BreadcrumbModel` struct (segments vec, active index, separator char)
- [ ] 7.2 Implement navigation (move_left, move_right, clamped), select returns (index, label)
- [ ] 7.3 Implement left-truncation logic (drop leading segments, prepend ellipsis when total exceeds width)
- [ ] 7.4 Define `BreadcrumbStyle` struct with builder methods
- [ ] 7.5 Implement breadcrumb render function — segments with separators, active segment highlight, truncation
- [ ] 7.6 Tests: truncation from left, single oversized segment, navigation clamping, select return value

## 8. Tooltip

- [ ] 8.1 Implement `Tooltip` struct with builder methods (anchor, content, max_width, border type, styles)
- [ ] 8.2 Implement vertical flip logic (render above anchor when below would overflow viewport)
- [ ] 8.3 Implement width computation from content, line wrapping at max_width
- [ ] 8.4 Implement tooltip render function — Clear + border + content text
- [ ] 8.5 Tests: vertical flip, width from content, max width wrapping, no border mode

## 9. Split Pane

- [ ] 9.1 Define `DividerPos` enum (Ratio, FromStart, FromEnd)
- [ ] 9.2 Define `SplitPaneModel` struct (divider pos, orientation, min_first, min_second)
- [ ] 9.3 Implement divider position computation — convert all DividerPos variants to absolute cell offsets, apply min constraints
- [ ] 9.4 Implement move_divider(delta) with clamping
- [ ] 9.5 Define `SplitPaneStyle` struct with builder (divider character, divider style)
- [ ] 9.6 Implement split pane render function — divider line, Clear not needed, return SplitRegions { first, second }
- [ ] 9.7 Tests: ratio computation, fixed from start/end, min size clamping, both minimums conflict, move clamped

## 10. Dialog Frame

- [ ] 10.1 Define `DialogModel` struct (title, buttons vec, focused button index, size config, dimming flag)
- [ ] 10.2 Implement button focus navigation (move_left, move_right, clamped), activate returns (index, label)
- [ ] 10.3 Define `DialogStyle` struct with builder (border, title style, button styles, dim style)
- [ ] 10.4 Implement dialog size computation (fixed or percentage of viewport)
- [ ] 10.5 Implement dialog render function — dim viewport (optional), centered border, title bar row, button row, return content Rect
- [ ] 10.6 Tests: centering computation, content rect dimensions, button focus navigation, dimming on/off

## 11. Integration

- [ ] 11.1 Verify `cargo check` passes for rat-chrome and the workspace
- [ ] 11.2 Verify `cargo test` passes for rat-chrome
- [ ] 11.3 Add crate-level doc comments with overview and per-module usage examples
