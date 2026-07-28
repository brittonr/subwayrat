# rat-keymap

`rat-keymap` provides a generic modal keymap for Ratatui applications.

It accepts any action type `A` and mode type `M`. It resolves key events, applies configuration overrides, and generates help text.

## Features

- **Generic over action and mode types**: Use any enum for actions and modes
- **Modal support**: Different key bindings per mode (e.g., Normal vs Insert)
- **String-based overrides**: Allow users to customize key bindings via config
- **Key parsing**: Parse human-readable key strings like "Ctrl+K", "Alt+Enter"
- **Help generation**: Generate help text showing all bindings for a mode
- **Function key support**: Handles F1-F12 function keys

## Example

```rust
use rat_keymap::{Keymap, KeyCombo};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum Mode { Normal, Insert }

#[derive(Debug, Clone, PartialEq, Eq)]
enum Action { Quit, Submit, MoveUp }

// Build a keymap with some initial bindings
let bindings = vec![
    (Mode::Normal, {
        let mut map = HashMap::new();
        map.insert(KeyCombo::new(KeyCode::Char('q'), false, false, false), Action::Quit);
        map.insert(KeyCombo::new(KeyCode::Char('k'), false, false, false), Action::MoveUp);
        map
    }),
    (Mode::Insert, {
        let mut map = HashMap::new();
        map.insert(KeyCombo::new(KeyCode::Enter, false, false, false), Action::Submit);
        map
    }),
];

let parse_action = |s: &str| match s {
    "quit" => Some(Action::Quit),
    "submit" => Some(Action::Submit),
    "move_up" => Some(Action::MoveUp),
    _ => None,
};

// Add user overrides
let overrides = vec![
    (Mode::Normal, {
        let mut map = HashMap::new();
        map.insert("Ctrl+c".to_string(), "quit".to_string());
        map
    }),
];

let keymap = Keymap::build(bindings, &overrides, parse_action);

// Resolve a key event
let event = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE);
let action = keymap.resolve(&Mode::Normal, &event);
assert_eq!(action, Some(Action::Quit));

// Get help text
let help = keymap.describe(&Mode::Normal);
println!("Normal mode bindings: {:?}", help);
```

## API

### `Keymap<A, M>`

The main keymap type, parameterized over:
- `A`: Action type (must be `Clone`)
- `M`: Mode type (must be `Eq + Hash + Clone`)

### Methods

- `resolve(&self, mode: &M, event: &KeyEvent) -> Option<A>`: Resolve a key event to an action
- `build(bindings, overrides, parse_action) -> Self`: Build from initial bindings with overrides
- `describe(&self, mode: &M) -> Vec<(String, A)>`: Get all bindings for a mode as help text
- `set(&mut self, mode: M, combo: KeyCombo, action: A)`: Add/update a single binding

### `KeyCombo`

Represents a key combination:
- `code: KeyCode`: The base key
- `ctrl: bool`: Whether Ctrl is held
- `alt: bool`: Whether Alt is held  
- `shift: bool`: Whether Shift is held

### Key string format

Supported formats for key strings:
- Simple keys: `"k"`, `"enter"`, `"esc"`
- With modifiers: `"Ctrl+c"`, `"Alt+F4"`, `"Ctrl+Shift+K"`
- Function keys: `"F1"` through `"F12"`
- Special names: `"space"`, `"backspace"`, `"delete"`, `"pageup"`, etc.

## License

MIT