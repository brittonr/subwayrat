//! Generic modal keymap implementation.

use std::collections::HashMap;
use std::hash::Hash;

use ratatui::crossterm::event::KeyEvent;

use crate::combo::{KeyCombo, format_key_combo, parse_key_string};

/// Generic modal keymap that maps key combinations to actions within specific modes.
///
/// Type parameters:
/// - `A`: Action type (must be Clone)
/// - `M`: Mode type (must be Eq + Hash + Clone)
#[derive(Debug, Clone)]
pub struct Keymap<A, M>
where
    A: Clone,
    M: Eq + Hash + Clone,
{
    /// Binding tables per mode.
    modes: HashMap<M, HashMap<KeyCombo, A>>,
}

impl<A, M> Keymap<A, M>
where
    A: Clone,
    M: Eq + Hash + Clone,
{
    /// Create a new empty keymap.
    pub fn new() -> Self {
        Self {
            modes: HashMap::new(),
        }
    }

    /// Resolve a key event in the given mode. Returns `None` for unmapped keys.
    pub fn resolve(&self, mode: &M, event: &KeyEvent) -> Option<A> {
        let combo = KeyCombo::from_event(event);
        self.modes
            .get(mode)
            .and_then(|mode_map| mode_map.get(&combo))
            .cloned()
    }

    /// Build a keymap from mode bindings with optional string-based overrides.
    ///
    /// # Parameters
    /// - `mode_bindings`: Initial bindings per mode
    /// - `overrides`: String-based overrides per mode (key_string -> action_string)
    /// - `parse_action`: Function to parse action strings
    pub fn build<F>(
        mode_bindings: Vec<(M, HashMap<KeyCombo, A>)>,
        overrides: &[(M, HashMap<String, String>)],
        parse_action: F,
    ) -> Self
    where
        F: Fn(&str) -> Option<A>,
    {
        let mut keymap = Self::new();

        // Apply initial bindings
        for (mode, bindings) in mode_bindings {
            keymap.modes.insert(mode, bindings);
        }

        // Apply overrides
        for (mode, mode_overrides) in overrides {
            let mode_map = keymap.modes.entry(mode.clone()).or_default();
            for (key_str, action_str) in mode_overrides {
                if let (Some(combo), Some(action)) =
                    (parse_key_string(key_str), parse_action(action_str))
                {
                    mode_map.insert(combo, action);
                }
            }
        }

        keymap
    }

    /// List all bindings for a mode (for help display).
    /// Returns tuples of (key_string, action).
    pub fn describe(&self, mode: &M) -> Vec<(String, A)> {
        self.modes
            .get(mode)
            .map(|mode_map| {
                mode_map
                    .iter()
                    .map(|(combo, action)| (format_key_combo(combo), action.clone()))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Add or update a single binding.
    pub fn set(&mut self, mode: M, combo: KeyCombo, action: A) {
        self.modes.entry(mode).or_default().insert(combo, action);
    }

    /// Get all modes that have bindings.
    pub fn modes(&self) -> impl Iterator<Item = &M> {
        self.modes.keys()
    }

    /// Check if a mode has any bindings.
    pub fn has_mode(&self, mode: &M) -> bool {
        self.modes.contains_key(mode)
    }

    /// Get the number of bindings in a specific mode.
    pub fn mode_binding_count(&self, mode: &M) -> usize {
        self.modes.get(mode).map(|m| m.len()).unwrap_or(0)
    }

    /// Clear all bindings for a mode.
    pub fn clear_mode(&mut self, mode: &M) {
        self.modes.remove(mode);
    }
}

impl<A, M> Default for Keymap<A, M>
where
    A: Clone,
    M: Eq + Hash + Clone,
{
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    enum TestMode {
        Normal,
        Insert,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    enum TestAction {
        MoveUp,
        MoveDown,
        Submit,
        Quit,
    }

    fn test_parse_action(s: &str) -> Option<TestAction> {
        match s {
            "move_up" => Some(TestAction::MoveUp),
            "move_down" => Some(TestAction::MoveDown),
            "submit" => Some(TestAction::Submit),
            "quit" => Some(TestAction::Quit),
            _ => None,
        }
    }

    #[test]
    fn basic_resolve() {
        let mut keymap = Keymap::new();
        keymap.set(
            TestMode::Normal,
            KeyCombo::new(KeyCode::Char('k'), false, false, false),
            TestAction::MoveUp,
        );

        let event = KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE);
        assert_eq!(
            keymap.resolve(&TestMode::Normal, &event),
            Some(TestAction::MoveUp)
        );

        // Different mode should return None
        assert_eq!(keymap.resolve(&TestMode::Insert, &event), None);

        // Different key should return None
        let other_event = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE);
        assert_eq!(keymap.resolve(&TestMode::Normal, &other_event), None);
    }

    #[test]
    fn build_with_overrides() {
        let mode_bindings = vec![
            (TestMode::Normal, {
                let mut map = HashMap::new();
                map.insert(
                    KeyCombo::new(KeyCode::Char('k'), false, false, false),
                    TestAction::MoveUp,
                );
                map.insert(
                    KeyCombo::new(KeyCode::Char('j'), false, false, false),
                    TestAction::MoveDown,
                );
                map
            }),
            (TestMode::Insert, {
                let mut map = HashMap::new();
                map.insert(
                    KeyCombo::new(KeyCode::Enter, false, false, false),
                    TestAction::Submit,
                );
                map
            }),
        ];

        let overrides = vec![(TestMode::Normal, {
            let mut map = HashMap::new();
            map.insert("q".to_string(), "quit".to_string());
            map
        })];

        let keymap = Keymap::build(mode_bindings, &overrides, test_parse_action);

        // Test original binding
        let k_event = KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE);
        assert_eq!(
            keymap.resolve(&TestMode::Normal, &k_event),
            Some(TestAction::MoveUp)
        );

        // Test override
        let q_event = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE);
        assert_eq!(
            keymap.resolve(&TestMode::Normal, &q_event),
            Some(TestAction::Quit)
        );

        // Test insert mode binding
        let enter_event = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        assert_eq!(
            keymap.resolve(&TestMode::Insert, &enter_event),
            Some(TestAction::Submit)
        );
    }

    #[test]
    fn describe() {
        let mut keymap = Keymap::new();
        keymap.set(
            TestMode::Normal,
            KeyCombo::new(KeyCode::Char('k'), false, false, false),
            TestAction::MoveUp,
        );
        keymap.set(
            TestMode::Normal,
            KeyCombo::new(KeyCode::Char('q'), true, false, false),
            TestAction::Quit,
        );

        let bindings = keymap.describe(&TestMode::Normal);
        assert_eq!(bindings.len(), 2);

        // Check that we have the expected bindings (order may vary due to HashMap)
        let binding_map: HashMap<String, TestAction> = bindings.into_iter().collect();
        assert_eq!(binding_map.get("k"), Some(&TestAction::MoveUp));
        assert_eq!(binding_map.get("Ctrl+q"), Some(&TestAction::Quit));

        // Empty mode should return empty list
        let empty_bindings = keymap.describe(&TestMode::Insert);
        assert!(empty_bindings.is_empty());
    }

    #[test]
    fn mode_management() {
        let mut keymap = Keymap::new();

        // Initially no modes
        assert_eq!(keymap.modes().count(), 0);
        assert!(!keymap.has_mode(&TestMode::Normal));
        assert_eq!(keymap.mode_binding_count(&TestMode::Normal), 0);

        // Add a binding
        keymap.set(
            TestMode::Normal,
            KeyCombo::new(KeyCode::Char('k'), false, false, false),
            TestAction::MoveUp,
        );

        assert_eq!(keymap.modes().count(), 1);
        assert!(keymap.has_mode(&TestMode::Normal));
        assert_eq!(keymap.mode_binding_count(&TestMode::Normal), 1);

        // Clear the mode
        keymap.clear_mode(&TestMode::Normal);
        assert_eq!(keymap.modes().count(), 0);
        assert!(!keymap.has_mode(&TestMode::Normal));
    }

    #[test]
    fn modifier_keys() {
        let mut keymap = Keymap::new();
        keymap.set(
            TestMode::Normal,
            KeyCombo::new(KeyCode::Char('c'), true, false, false),
            TestAction::Quit,
        );

        let ctrl_c = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
        assert_eq!(
            keymap.resolve(&TestMode::Normal, &ctrl_c),
            Some(TestAction::Quit)
        );

        let plain_c = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::NONE);
        assert_eq!(keymap.resolve(&TestMode::Normal, &plain_c), None);
    }
}
