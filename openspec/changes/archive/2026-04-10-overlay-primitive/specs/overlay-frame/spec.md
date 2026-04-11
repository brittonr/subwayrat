## ADDED Requirements

### Requirement: Overlay frame computes anchored placement from viewport-relative sizing
The system SHALL provide an overlay frame primitive that computes an outer overlay `Rect` from a viewport `Rect`, an anchor position, width, height, and optional x/y offsets. Width and height SHALL each support either fixed cell sizes or percentages of the viewport.

#### Scenario: Center a fixed-size overlay
- **WHEN** the viewport is 80×24 and the overlay width is 40, height is 10, with center anchoring
- **THEN** the computed outer rect is centered within the viewport

#### Scenario: Anchor a percentage-sized overlay to the right edge
- **WHEN** the viewport is 100×40 and the overlay width is 30%, height is 100%, with right-edge anchoring
- **THEN** the computed outer rect is 30 columns wide, 40 rows tall, and flush to the right side of the viewport

#### Scenario: Clamp oversized overlay requests
- **WHEN** the requested overlay width or height exceeds the viewport
- **THEN** the computed outer rect is clamped so it stays within the viewport bounds

### Requirement: Overlay frame returns a child content rect for caller-rendered body content
The system SHALL render overlay chrome without owning the body widget content. The overlay render API SHALL return layout information containing the computed outer rect and the inner content rect that remains after applying any border or title chrome.

#### Scenario: Border chrome reduces inner content area
- **WHEN** the overlay renders with a full border around a 40×10 outer rect
- **THEN** the returned inner content rect excludes the border cells on each side

#### Scenario: Borderless overlay uses full area as content area
- **WHEN** the overlay renders without border chrome
- **THEN** the returned inner content rect matches the computed outer rect

### Requirement: Overlay frame optionally clears and dims the covered region before rendering chrome
The system SHALL support clearing the overlay region before drawing overlay chrome, and it SHALL optionally support dimming or styling the background viewport outside the overlay rect.

#### Scenario: Clear overlay region before drawing
- **WHEN** clear behavior is enabled and the overlay is rendered over previously drawn content
- **THEN** cells within the overlay outer rect are reset before border, fill, and title styling are applied

#### Scenario: Dim the background outside the overlay
- **WHEN** backdrop dimming is enabled
- **THEN** viewport cells outside the overlay outer rect are overwritten with the configured dim/background style before the overlay frame is drawn

#### Scenario: Leave background untouched when dimming is disabled
- **WHEN** backdrop dimming is disabled
- **THEN** rendering the overlay does not modify cells outside the overlay outer rect
