## ADDED Requirements

### Requirement: Viewport clips to visible region
The layout engine SHALL compute which windows are fully or partially visible given a viewport size and scroll offset. The output SHALL include a clipped Rect for each visible window in viewport-local coordinates (0,0 at viewport top-left).

#### Scenario: Window fully inside viewport
- **WHEN** a window has strip-space rect (x=10, y=0, w=20, h=15) and viewport is (offset=5, width=80, height=20)
- **THEN** the window appears in viewport at (x=5, y=0, w=20, h=15) and is marked fully visible

#### Scenario: Window partially visible
- **WHEN** a window has strip-space rect (x=0, y=0, w=30, h=10) and viewport offset is 20
- **THEN** the visible portion is (x=0, y=0, w=10, h=10) in viewport coordinates, representing the rightmost 10 columns of the window

#### Scenario: Window entirely outside viewport
- **WHEN** a window has strip-space rect (x=100, y=0, w=20, h=10) and viewport is (offset=0, width=80)
- **THEN** the window is not included in the visible output

### Requirement: Focus-driven scroll positioning
When a focus target is set, the viewport scroll offset SHALL adjust so the focused window is visible. If the focused window is already fully visible, the offset SHALL NOT change. If the focused window is not visible, the offset SHALL center the window in the viewport when possible.

#### Scenario: Focus moves to off-screen window right
- **WHEN** focus moves to a window at strip-space x=200 with width 30, and the viewport is at offset 0 with width 80
- **THEN** the viewport offset shifts so the window is centered: offset ≈ 200 + 15 - 40 = 175

#### Scenario: Focus on already-visible window
- **WHEN** focus is on a window at strip-space x=20 with width 30, and viewport is at offset 10 with width 80
- **THEN** the viewport offset remains 10 (window is already fully visible at viewport x=10)

#### Scenario: Centering clamped at strip start
- **WHEN** focus moves to the first column at strip-space x=0
- **THEN** the viewport offset is clamped to 0 (cannot scroll before the start)

#### Scenario: Centering clamped at strip end
- **WHEN** focus moves to the last column and centering would scroll past the strip's total width
- **THEN** the viewport offset is clamped so the viewport does not extend beyond the strip's content

### Requirement: Manual scroll offset override
The consumer SHALL be able to set an explicit scroll offset, bypassing focus-driven positioning. When a manual offset is set, focus changes SHALL NOT automatically adjust the viewport until the consumer re-enables focus tracking.

#### Scenario: Manual offset set
- **WHEN** the consumer sets scroll offset to 50
- **THEN** the viewport renders from offset 50 regardless of which window has focus

#### Scenario: Re-enable focus tracking
- **WHEN** the consumer clears the manual offset override
- **THEN** the next focus change triggers viewport repositioning as normal

### Requirement: Scroll offset reported in layout result
The layout result SHALL include the computed scroll offset along the primary axis. This allows the consumer to render scroll indicators or position decorations.

#### Scenario: Offset available after layout
- **WHEN** compute_layout returns a result
- **THEN** the result contains the scroll offset as a u16 value and the total strip extent along the primary axis
