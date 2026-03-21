## ADDED Requirements

### Requirement: Render-free search state
`OutputSearch` SHALL have no ratatui dependency. It SHALL hold query text, match list, current match index, search mode, and scroll-to-current flag. Methods for match computation (`update_matches`, `find_substring_matches`, `find_fuzzy_matches`), navigation (`next_match`, `prev_match`), and input (`type_char`, `backspace`, `toggle_mode`, `activate`, `deactivate`, `cancel`) SHALL remain on the struct. All rendering SHALL be removed from the struct.

#### Scenario: Search state compiles without ratatui
- **WHEN** `OutputSearch` is compiled
- **THEN** it compiles without the `ratatui` crate in scope

#### Scenario: Match computation works independently
- **WHEN** `update_matches()` is called with plain text lines and query "hello"
- **THEN** `matches` is populated with row/byte_start/byte_end entries without any rendering types involved

### Requirement: Standalone search overlay rendering
A free function `render_search_overlay()` SHALL accept `&OutputSearch`, `&mut Frame`, and `Rect`, and render the search popup. It SHALL live in a separate module or file from the search state struct.

#### Scenario: Render overlay from search reference
- **WHEN** `render_search_overlay(&search, frame, area)` is called with an active search
- **THEN** a popup is rendered showing the mode label, query text, cursor, and match count

### Requirement: Highlight application remains standalone
`apply_search_highlights()` SHALL remain a free function. It MAY live alongside the render function or in its own module. Its signature SHALL continue to accept `&OutputSearch` by reference.

#### Scenario: Highlights applied without owning search state
- **WHEN** `apply_search_highlights(lines, plain_lines, &search, match_style, current_style)` is called
- **THEN** matching spans are restyled in the lines array
