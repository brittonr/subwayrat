## ADDED Requirements

### Requirement: FuzzyCandidate type
The system SHALL define a `FuzzyCandidate` struct containing: `id` (String), `text` (String, the searchable content), `context` (Option<String>, secondary info like file path or heading breadcrumb), `icon` (Option<char>). The candidate list is provided by the caller through a trait.

#### Scenario: Candidate with context
- **WHEN** a FuzzyCandidate is created with text="Ship docs" and context=Some("projects.org > Q2")
- **THEN** both fields are accessible for display

#### Scenario: Candidate without context
- **WHEN** a FuzzyCandidate has context=None
- **THEN** only the text is rendered, no secondary line

### Requirement: FuzzySource trait
The system SHALL define a trait `FuzzySource` with method `fn candidates(&self) -> &[FuzzyCandidate]`. The widget calls this to obtain the full candidate list. The trait SHALL be object-safe. A blanket impl SHALL exist for `Vec<FuzzyCandidate>`.

#### Scenario: Vec as source
- **WHEN** a `Vec<FuzzyCandidate>` with 100 items is used as a FuzzySource
- **THEN** `candidates()` returns a slice of length 100

### Requirement: Fuzzy scoring
The system SHALL score each candidate against the query using a character-matching algorithm that rewards: consecutive character matches, matches at word boundaries, matches at the start of the string. Candidates with score 0 (no match) SHALL be filtered out. Results SHALL be sorted by score descending, then alphabetically for ties.

#### Scenario: Exact prefix match scores highest
- **WHEN** query is "ship" and candidates are ["Ship docs", "Worship plan", "Flagship"]
- **THEN** "Ship docs" has the highest score (prefix match + word boundary)

#### Scenario: No match filtered
- **WHEN** query is "xyz" and no candidate contains those characters in order
- **THEN** the filtered result list is empty

#### Scenario: Empty query shows all
- **WHEN** the query is an empty string
- **THEN** all candidates are shown (unfiltered), in their original order

### Requirement: Incremental search input
The system SHALL provide a text input at the top of the finder. Each keystroke SHALL re-score and re-sort the candidate list. The input SHALL show a prompt indicator (e.g., `> `) and the current query text. Backspace removes characters. Escape clears the query (if non-empty) or closes the finder (if already empty).

#### Scenario: Type to filter
- **WHEN** the user types "pro" into the search input
- **THEN** only candidates matching "pro" are shown in the results list

#### Scenario: Backspace narrows less
- **WHEN** the query is "pro" and the user presses backspace
- **THEN** the query becomes "pr" and the result list updates (more items may appear)

#### Scenario: Escape with query clears first
- **WHEN** Escape is pressed with query "abc"
- **THEN** the query clears to "" and all candidates are shown; the finder stays open

#### Scenario: Escape with empty query closes
- **WHEN** Escape is pressed with an empty query
- **THEN** the finder closes with no selection

### Requirement: Result list navigation
The system SHALL display filtered candidates in a scrollable list below the input. Arrow up/down SHALL move the selection. The selected candidate SHALL be highlighted. The list SHALL scroll to keep the selection visible. The first item SHALL be auto-selected when results change.

#### Scenario: Navigate down
- **WHEN** the selection is on item 0 and down-arrow is pressed
- **THEN** the selection moves to item 1

#### Scenario: Scroll follows selection
- **WHEN** the list shows 10 visible rows and the user navigates to item 15
- **THEN** the list scrolls so item 15 is visible

#### Scenario: Results change resets selection
- **WHEN** the user types a character and the result list changes
- **THEN** the selection resets to item 0

### Requirement: Match highlighting
The system SHALL highlight the matched characters within each candidate's text. Matched characters SHALL use a distinct style (bold or different color via `FuzzyStyle`). Non-matched characters use the base style.

#### Scenario: Highlight matches
- **WHEN** query is "sd" and candidate text is "Ship docs"
- **THEN** the 'S' and 'd' characters are rendered in the match highlight style

### Requirement: Confirm and cancel
The system SHALL return a selection result on Enter (the currently highlighted candidate) or None on final Escape. `FuzzyState::take_result()` SHALL return `Option<FuzzyCandidate>`.

#### Scenario: Confirm selection
- **WHEN** Enter is pressed with item "Ship docs" selected
- **THEN** `take_result()` returns Some(candidate) with id matching "Ship docs"

#### Scenario: Cancel returns none
- **WHEN** the finder is closed via Escape with empty query
- **THEN** `take_result()` returns None

### Requirement: FuzzyState and Action enum
The system SHALL expose `FuzzyState` containing: query string, source reference, scored/filtered results, selected index, scroll offset, open/closed flag, and result. An `Action` enum SHALL include: `Open`, `Close`, `TypeChar(char)`, `Backspace`, `SelectNext`, `SelectPrev`, `Confirm`, `Cancel`. A `handle_action(state, action)` function SHALL process actions.

#### Scenario: Handle TypeChar
- **WHEN** `handle_action(state, Action::TypeChar('a'))` is called
- **THEN** query becomes "a" (appended), results are re-scored, selection resets to 0

### Requirement: Fuzzy rendering
The system SHALL implement `StatefulWidget` for ratatui. The widget SHALL render as a bordered box showing the input line at the top, a result count indicator, and the scrollable candidate list below. A `FuzzyStyle` struct SHALL configure: input prompt style, query text style, match highlight style, selected item style, context line style, border style, and result count style.

#### Scenario: Render with results
- **WHEN** the finder has query "pro" matching 5 of 50 candidates
- **THEN** the widget shows "> pro" input line, "5/50" count, and up to visible-height candidate rows
