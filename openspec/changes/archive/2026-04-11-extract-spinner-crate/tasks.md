## 1. Create the spinner crate

- [x] 1.1 Add `crates/rat-spinner` to the workspace and create its `Cargo.toml`
- [x] 1.2 Move spinner presets and playback state into `rat-spinner`
- [x] 1.3 Add unit tests in `rat-spinner` for preset, custom, reverse, timed, and empty-frame behavior

## 2. Rewire the loader widget

- [x] 2.1 Remove the backward-compatible owned `rat-widgets::Loader` wrapper API
- [x] 2.2 Update `rat-widgets::Loader` to use `rat_spinner::{SpinnerSpec, SpinnerState}` directly
- [x] 2.3 Update `rat-widgets` exports and tests for the new loader API

## 3. Update the showcase

- [x] 3.1 Add a direct `rat-spinner` dependency to `crates/showcase`
- [x] 3.2 Update the loader showcase to use spinner state from `rat-spinner`
- [x] 3.3 Keep the custom-frame spinner example in the showcase and verify the refactor with `cargo test -p rat-spinner -p rat-widgets` and `cargo check -p showcase`
