//! Key combination parsing and formatting.

use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// A single key combination (e.g. `Ctrl+Shift+K`).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct KeyCombo {
    pub code: KeyCode,
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
}

impl KeyCombo {
    /// Create a new KeyCombo.
    pub fn new(code: KeyCode, ctrl: bool, alt: bool, shift: bool) -> Self {
        Self {
            code,
            ctrl,
            alt,
            shift,
        }
    }

    /// Create a KeyCombo from a crossterm KeyEvent.
    pub fn from_event(event: &KeyEvent) -> Self {
        Self {
            code: event.code,
            ctrl: event.modifiers.contains(KeyModifiers::CONTROL),
            alt: event.modifiers.contains(KeyModifiers::ALT),
            shift: event.modifiers.contains(KeyModifiers::SHIFT),
        }
    }
}

// Key string parser lookup table
const KEY_CODE_NAMES: &[(&str, KeyCode)] = &[
    ("enter", KeyCode::Enter),
    ("return", KeyCode::Enter),
    ("cr", KeyCode::Enter),
    ("esc", KeyCode::Esc),
    ("escape", KeyCode::Esc),
    ("tab", KeyCode::Tab),
    ("backspace", KeyCode::Backspace),
    ("bs", KeyCode::Backspace),
    ("delete", KeyCode::Delete),
    ("del", KeyCode::Delete),
    ("up", KeyCode::Up),
    ("down", KeyCode::Down),
    ("left", KeyCode::Left),
    ("right", KeyCode::Right),
    ("home", KeyCode::Home),
    ("end", KeyCode::End),
    ("pageup", KeyCode::PageUp),
    ("pgup", KeyCode::PageUp),
    ("pagedown", KeyCode::PageDown),
    ("pgdn", KeyCode::PageDown),
    ("space", KeyCode::Char(' ')),
    ("spc", KeyCode::Char(' ')),
    ("/", KeyCode::Char('/')),
];

/// Parse a human-readable key string like `"Ctrl+K"`, `"Alt+Enter"`, `"e"`.
pub fn parse_key_string(s: &str) -> Option<KeyCombo> {
    let parts: Vec<&str> = s.split('+').map(str::trim).collect();
    let key_str = parts.last()?;

    let mut ctrl = false;
    let mut alt = false;
    let mut shift = false;

    for part in &parts[..parts.len() - 1] {
        match part.to_lowercase().as_str() {
            "ctrl" => ctrl = true,
            "alt" => alt = true,
            "shift" => shift = true,
            _ => {}
        }
    }

    let key_lower = key_str.to_lowercase();
    let code = KEY_CODE_NAMES
        .iter()
        .find(|(name, _)| *name == key_lower)
        .map(|(_, code)| *code)
        .or_else(|| {
            if key_str.len() == 1 {
                key_str.chars().next().map(KeyCode::Char)
            } else if key_str.to_lowercase().starts_with('f') && key_str.len() > 1 {
                // Handle function keys F1-F12
                if let Ok(num) = key_str[1..].parse::<u8>()
                    && (1..=12).contains(&num)
                {
                    return Some(KeyCode::F(num));
                }
                None
            } else {
                None
            }
        })?;

    Some(KeyCombo {
        code,
        ctrl,
        alt,
        shift,
    })
}

/// Format a KeyCombo into a human-readable string.
pub fn format_key_combo(k: &KeyCombo) -> String {
    let mut parts = Vec::new();
    if k.ctrl {
        parts.push("Ctrl".to_string());
    }
    if k.alt {
        parts.push("Alt".to_string());
    }
    if k.shift {
        parts.push("Shift".to_string());
    }
    parts.push(match k.code {
        KeyCode::Char(' ') => "Space".to_string(),
        KeyCode::Char(c) => c.to_string(),
        KeyCode::Enter => "Enter".to_string(),
        KeyCode::Esc => "Esc".to_string(),
        KeyCode::Tab => "Tab".to_string(),
        KeyCode::Backspace => "Backspace".to_string(),
        KeyCode::Delete => "Delete".to_string(),
        KeyCode::Up => "Up".to_string(),
        KeyCode::Down => "Down".to_string(),
        KeyCode::Left => "Left".to_string(),
        KeyCode::Right => "Right".to_string(),
        KeyCode::Home => "Home".to_string(),
        KeyCode::End => "End".to_string(),
        KeyCode::PageUp => "PageUp".to_string(),
        KeyCode::PageDown => "PageDown".to_string(),
        KeyCode::F(n) => format!("F{}", n),
        other => format!("{:?}", other),
    });
    parts.join("+")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_simple_key() {
        let combo = parse_key_string("k").unwrap();
        assert_eq!(combo.code, KeyCode::Char('k'));
        assert!(!combo.ctrl && !combo.alt && !combo.shift);
    }

    #[test]
    fn parse_ctrl_key() {
        let combo = parse_key_string("Ctrl+c").unwrap();
        assert_eq!(combo.code, KeyCode::Char('c'));
        assert!(combo.ctrl && !combo.alt && !combo.shift);
    }

    #[test]
    fn parse_complex_combo() {
        let combo = parse_key_string("Ctrl+Shift+F1").unwrap();
        assert_eq!(combo.code, KeyCode::F(1));
        assert!(combo.ctrl && !combo.alt && combo.shift);
    }

    #[test]
    fn parse_special_keys() {
        assert_eq!(parse_key_string("enter").unwrap().code, KeyCode::Enter);
        assert_eq!(parse_key_string("esc").unwrap().code, KeyCode::Esc);
        assert_eq!(parse_key_string("space").unwrap().code, KeyCode::Char(' '));
    }

    #[test]
    fn format_simple_key() {
        let combo = KeyCombo::new(KeyCode::Char('q'), false, false, false);
        assert_eq!(format_key_combo(&combo), "q");
    }

    #[test]
    fn format_ctrl_key() {
        let combo = KeyCombo::new(KeyCode::Char('c'), true, false, false);
        assert_eq!(format_key_combo(&combo), "Ctrl+c");
    }

    #[test]
    fn format_complex_combo() {
        let combo = KeyCombo::new(KeyCode::F(1), true, false, true);
        assert_eq!(format_key_combo(&combo), "Ctrl+Shift+F1");
    }

    #[test]
    fn from_event() {
        let event = KeyEvent::new(KeyCode::Char('k'), KeyModifiers::CONTROL);
        let combo = KeyCombo::from_event(&event);
        assert_eq!(combo.code, KeyCode::Char('k'));
        assert!(combo.ctrl && !combo.alt && !combo.shift);
    }
}
