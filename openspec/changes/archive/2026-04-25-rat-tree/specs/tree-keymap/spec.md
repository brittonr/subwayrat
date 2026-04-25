## ADDED Requirements

### Requirement: TreeAction enum covers all navigation operations
The crate SHALL define a `TreeAction` enum with variants: `Up`, `Down`, `Expand`, `Collapse`, `Toggle`, `Parent`, `FirstChild`, `NextSibling`, `PrevSibling`, `First`, `Last`, `PageUp`, `PageDown`, `Select`.

#### Scenario: Enum is exhaustive for navigation
- **WHEN** a consumer matches on `TreeAction`
- **THEN** all listed variants SHALL be available.

#### Scenario: Enum derives required traits
- **WHEN** `TreeAction` is used with `rat-keymap`
- **THEN** it SHALL derive `Clone`, `Debug`, `PartialEq`, and `Eq`.

### Requirement: Default keymap with vim-style bindings
The crate SHALL provide a `default_keymap()` function returning `Keymap<TreeAction, ()>` with single-mode (unit mode) vim-style bindings.

#### Scenario: Default movement keys
- **WHEN** the default keymap is queried
- **THEN** `k`/`Up` SHALL map to `Up`, `j`/`Down` SHALL map to `Down`, `g` SHALL map to `First`, `G` SHALL map to `Last`.

#### Scenario: Default expand/collapse keys
- **WHEN** the default keymap is queried
- **THEN** `l`/`Right` SHALL map to `Expand`, `h`/`Left` SHALL map to `Collapse`, `Space` SHALL map to `Toggle`.

#### Scenario: Default structural navigation keys
- **WHEN** the default keymap is queried
- **THEN** `p` SHALL map to `Parent`, `Enter` SHALL map to `FirstChild`, `J` SHALL map to `NextSibling`, `K` SHALL map to `PrevSibling`.

#### Scenario: Default page keys
- **WHEN** the default keymap is queried
- **THEN** `Ctrl+d`/`PageDown` SHALL map to `PageDown`, `Ctrl+u`/`PageUp` SHALL map to `PageUp`.

#### Scenario: Default select key
- **WHEN** the default keymap is queried
- **THEN** `Enter` SHALL map to `Select` when on a leaf node (consumers handle the dual purpose of Enter contextually), or consumers can rebind.

### Requirement: parse_tree_action for user overrides
The crate SHALL provide a `parse_tree_action(s: &str) -> Option<TreeAction>` function that maps snake_case strings to `TreeAction` variants, suitable for use with `Keymap::build` overrides.

#### Scenario: Known action string
- **WHEN** `parse_tree_action("next_sibling")` is called
- **THEN** it SHALL return `Some(TreeAction::NextSibling)`.

#### Scenario: Unknown action string
- **WHEN** `parse_tree_action("nonexistent")` is called
- **THEN** it SHALL return `None`.

### Requirement: Consumer can supply custom keymap
Consumers SHALL be able to construct their own `Keymap<TreeAction, M>` for any mode type `M` and pass it to the tree widget, replacing or extending the default bindings.

#### Scenario: Modal keymap with Normal and Insert modes
- **WHEN** a consumer builds a `Keymap<TreeAction, AppMode>` with different bindings per mode
- **THEN** `keymap.resolve(&AppMode::Normal, &event)` and `keymap.resolve(&AppMode::Insert, &event)` SHALL return mode-specific actions.

#### Scenario: Override a default binding
- **WHEN** a consumer builds the default keymap with an override mapping `k` to `Collapse`
- **THEN** pressing `k` SHALL resolve to `Collapse` instead of `Up`.
