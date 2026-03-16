//! Leader menu state: open/close, key handling, submenu navigation.

use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::types::{LeaderAction, LeaderMenuDef};

/// Leader key menu state and navigation.
///
/// Generic over `A`, the application's action type.
pub struct LeaderMenu<A> {
    /// Whether the overlay is visible.
    pub visible: bool,
    /// Stack of menu levels (root at bottom, current at top).
    pub(crate) stack: Vec<LeaderMenuDef<A>>,
    /// Breadcrumb labels for the title bar.
    pub(crate) breadcrumb: Vec<String>,
    /// All submenu definitions, keyed by label.
    pub(crate) submenus: Vec<LeaderMenuDef<A>>,
    /// The root menu definition.
    pub(crate) root: LeaderMenuDef<A>,
}

impl<A> LeaderMenu<A> {
    /// Create from pre-built parts.
    pub(crate) fn from_parts(
        root: LeaderMenuDef<A>,
        submenus: Vec<LeaderMenuDef<A>>,
    ) -> Self {
        Self {
            visible: false,
            stack: Vec::new(),
            breadcrumb: Vec::new(),
            submenus,
            root,
        }
    }

    /// The root menu definition.
    pub fn root_def(&self) -> &LeaderMenuDef<A> {
        &self.root
    }

    /// All submenu definitions.
    pub fn submenu_defs(&self) -> &[LeaderMenuDef<A>] {
        &self.submenus
    }

    /// The currently displayed menu level.
    pub(crate) fn current(&self) -> Option<&LeaderMenuDef<A>> {
        self.stack.last()
    }

    /// Close the menu entirely.
    pub fn close(&mut self) {
        self.visible = false;
        self.stack.clear();
        self.breadcrumb.clear();
    }
}

impl<A: Clone> LeaderMenu<A> {
    /// Build from contributors (convenience wrapper around [`crate::build`]).
    pub fn build(
        contributors: &[&dyn crate::MenuContributor<A>],
        hidden: &crate::HiddenSet,
    ) -> crate::builder::BuildResult<A> {
        crate::builder::build(contributors, hidden)
    }

    /// Open the menu (shows root level).
    pub fn open(&mut self) {
        self.visible = true;
        self.stack.clear();
        self.breadcrumb.clear();
        self.stack.push(self.root.clone());
    }

    /// Handle a key press while the menu is visible.
    ///
    /// Returns `Some(action)` if an action should be dispatched,
    /// `None` if the key was consumed internally (submenu nav, close).
    pub fn handle_key(&mut self, key: &KeyEvent) -> Option<LeaderAction<A>> {
        if !self.visible {
            return None;
        }

        // Escape → back one level or close.
        if key.code == KeyCode::Esc {
            if self.stack.len() > 1 {
                self.stack.pop();
                self.breadcrumb.pop();
            } else {
                self.close();
            }
            return None;
        }

        // Match single character keys (with or without Shift).
        let ch = match key.code {
            KeyCode::Char(c)
                if key.modifiers.is_empty()
                    || key.modifiers == KeyModifiers::SHIFT =>
            {
                c
            }
            _ => {
                // Non-char key → dismiss (Helix behavior).
                self.close();
                return None;
            }
        };

        let current = match self.current() {
            Some(m) => m,
            None => {
                self.close();
                return None;
            }
        };

        // Find matching item.
        if let Some(item) = current.items.iter().find(|i| i.key == ch) {
            match &item.action {
                LeaderAction::Submenu(name) => {
                    if let Some(sub) =
                        self.submenus.iter().find(|s| s.label == *name)
                    {
                        self.breadcrumb.push(item.label.clone());
                        self.stack.push(sub.clone());
                    } else {
                        self.close();
                    }
                    None
                }
                action => {
                    let result = action.clone();
                    self.close();
                    Some(result)
                }
            }
        } else {
            // Unknown key → dismiss.
            self.close();
            None
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use ratatui::crossterm::event::KeyEventKind;

    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq)]
    enum Act {
        Save,
        Open,
    }

    fn make_menu() -> LeaderMenu<Act> {
        let root = LeaderMenuDef {
            label: "Leader".into(),
            items: vec![
                LeaderMenuItem {
                    key: 's',
                    label: "save".into(),
                    action: LeaderAction::Action(Act::Save),
                },
                LeaderMenuItem {
                    key: 'o',
                    label: "open".into(),
                    action: LeaderAction::Action(Act::Open),
                },
                LeaderMenuItem {
                    key: 'x',
                    label: "extra".into(),
                    action: LeaderAction::Submenu("extra".into()),
                },
            ],
        };
        let submenus = vec![LeaderMenuDef {
            label: "extra".into(),
            items: vec![LeaderMenuItem {
                key: 'a',
                label: "alpha".into(),
                action: LeaderAction::Command("/alpha".into()),
            }],
        }];
        LeaderMenu::from_parts(root, submenus)
    }

    fn key(c: char) -> KeyEvent {
        KeyEvent::new_with_kind(
            KeyCode::Char(c),
            KeyModifiers::NONE,
            KeyEventKind::Press,
        )
    }

    fn shift_key(c: char) -> KeyEvent {
        KeyEvent::new_with_kind(
            KeyCode::Char(c),
            KeyModifiers::SHIFT,
            KeyEventKind::Press,
        )
    }

    fn esc() -> KeyEvent {
        KeyEvent::new_with_kind(
            KeyCode::Esc,
            KeyModifiers::NONE,
            KeyEventKind::Press,
        )
    }

    use crate::types::LeaderMenuItem;

    #[test]
    fn opens_and_closes() {
        let mut m = make_menu();
        assert!(!m.visible);
        m.open();
        assert!(m.visible);
        m.close();
        assert!(!m.visible);
    }

    #[test]
    fn esc_closes_root() {
        let mut m = make_menu();
        m.open();
        let r = m.handle_key(&esc());
        assert!(r.is_none());
        assert!(!m.visible);
    }

    #[test]
    fn unknown_key_dismisses() {
        let mut m = make_menu();
        m.open();
        let r = m.handle_key(&key('z'));
        assert!(r.is_none());
        assert!(!m.visible);
    }

    #[test]
    fn direct_action_returns_and_closes() {
        let mut m = make_menu();
        m.open();
        let r = m.handle_key(&key('s'));
        assert_eq!(r, Some(LeaderAction::Action(Act::Save)));
        assert!(!m.visible);
    }

    #[test]
    fn submenu_navigation() {
        let mut m = make_menu();
        m.open();

        // Enter submenu.
        let r = m.handle_key(&key('x'));
        assert!(r.is_none());
        assert!(m.visible);
        assert_eq!(m.stack.len(), 2);

        // Select item in submenu.
        let r = m.handle_key(&key('a'));
        assert_eq!(r, Some(LeaderAction::Command("/alpha".into())));
        assert!(!m.visible);
    }

    #[test]
    fn esc_goes_back_from_submenu() {
        let mut m = make_menu();
        m.open();
        m.handle_key(&key('x'));
        assert_eq!(m.stack.len(), 2);

        // Back to root.
        m.handle_key(&esc());
        assert!(m.visible);
        assert_eq!(m.stack.len(), 1);

        // Close.
        m.handle_key(&esc());
        assert!(!m.visible);
    }

    #[test]
    fn not_visible_returns_none() {
        let mut m = make_menu();
        let r = m.handle_key(&key('s'));
        assert!(r.is_none());
    }

    #[test]
    fn shift_key_matches_uppercase() {
        let root = LeaderMenuDef {
            label: "Leader".into(),
            items: vec![LeaderMenuItem {
                key: 'T',
                label: "toggle".into(),
                action: LeaderAction::Action(Act::Save),
            }],
        };
        let mut m = LeaderMenu::from_parts(root, vec![]);
        m.open();
        let r = m.handle_key(&shift_key('T'));
        assert_eq!(r, Some(LeaderAction::Action(Act::Save)));
    }
}
