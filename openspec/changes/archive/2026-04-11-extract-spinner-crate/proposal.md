## Why

The spinner logic currently lives inside `rat-widgets`, mixes reusable animation state with the loader widget, and still carries a compatibility wrapper that the user now wants removed. Pulling spinner state into its own crate makes the animation model reusable across the workspace and lets `rat-widgets::Loader` become a focused widget again.

## What Changes

- Add a new `rat-spinner` crate for spinner presets, custom frame sets, intervals, reverse playback, and animation state.
- **BREAKING** Remove the owned backward-compatibility `rat-widgets::Loader` wrapper and keep only the stateful loader widget API.
- Update `rat-widgets::Loader` to depend on `rat-spinner` instead of owning spinner model types.
- Update `showcase` to depend on `rat-spinner` directly and demonstrate both preset and custom-frame spinners.

## Capabilities

### New Capabilities
- `spinner-animation`: Reusable spinner state and preset definitions for ratatui-oriented crates and demos.

### Modified Capabilities
- None.

## Impact

- New crate: `crates/rat-spinner`
- Modified crates: `crates/rat-widgets`, `crates/showcase`
- Workspace manifest updates in `Cargo.toml`
- Public API break in `rat-widgets::Loader`
