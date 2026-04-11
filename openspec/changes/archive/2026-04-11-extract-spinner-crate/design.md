## Context

`rat-widgets` currently contains both loader rendering and spinner state management. That makes the spinner types harder to reuse and forced a compatibility layer when the loader API was redesigned. The workspace already prefers focused crates, so spinner playback belongs in its own crate while `rat-widgets` keeps the ratatui rendering surface.

## Goals / Non-Goals

**Goals:**
- Move spinner presets, frame storage, intervals, and playback state into `rat-spinner`
- Keep `rat-widgets::Loader` as a small ratatui widget built on `rat-spinner`
- Update `showcase` to use `rat-spinner` directly for the loader demos, including a custom-frame example
- Remove the temporary backward-compatibility wrapper API from `rat-widgets::Loader`

**Non-Goals:**
- Add more presets than the current set
- Introduce real-time clocks or async animation drivers
- Change unrelated widget APIs

## Decisions

### 1. Put only animation primitives in `rat-spinner`
`rat-spinner` will own `SpinnerSpec`, `SpinnerFrames`, `SpinnerPreset`, and `SpinnerState`. It will stay free of ratatui so other crates can use it without pulling widget code.

Alternative considered: keep spinner types re-exported only from `rat-widgets`. Rejected because that still centers the reusable animation model inside a widget crate.

### 2. Make `rat-widgets::Loader` a stateless widget config
`Loader` becomes the stateless render configuration type. It accepts a `SpinnerSpec` and renders against an external `SpinnerState`.

Alternative considered: keep a `LoaderState` wrapper around `SpinnerState`. Rejected because it adds a thin type with no extra behavior.

### 3. Update showcase to depend on `rat-spinner`
The showcase will import spinner presets and state from `rat-spinner`, then feed them into `rat-widgets::Loader`. That proves the extraction worked and gives a direct example of custom spinner frames.

Alternative considered: rely on `rat-widgets` re-exports. Rejected because the point of the change is making the spinner crate visible and reusable.

## Risks / Trade-offs

- [Public API break] -> Mitigation: the user explicitly requested removing backward compatibility in this change.
- [More crate wiring] -> Mitigation: keep `rat-spinner` tiny and dependency-free.
- [Showcase behavior drift] -> Mitigation: keep compile checks and rat-widgets tests green after the refactor.
