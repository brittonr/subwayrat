//! Leader menu state — wraps ratcore::leaderkey::LeaderMenu with crossterm key conversion.

use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use ratcore::leaderkey::{self, LeaderAction, LeaderMenuDef, MenuInput};

/// Leader key menu state and navigation.
///
/// Thin wrapper around `ratcore::leaderkey::LeaderMenu` that converts
/// crossterm `KeyEvent` into the platform-neutral `MenuInput`.
pub struct LeaderMenu<A> {
    pub(crate) inner: leaderkey::LeaderMenu<A>,
}

impl<A> LeaderMenu<A> {
    /// Wrap a ratcore LeaderMenu.
    pub(crate) fn from_core(inner: leaderkey::LeaderMenu<A>) -> Self {
        Self { inner }
    }

    /// Whether the overlay is visible.
    pub fn visible(&self) -> bool {
        self.inner.visible
    }

    /// The root menu definition.
    pub fn root_def(&self) -> &LeaderMenuDef<A> {
        self.inner.root_def()
    }

    /// All submenu definitions.
    pub fn submenu_defs(&self) -> &[LeaderMenuDef<A>] {
        self.inner.submenu_defs()
    }

    /// The currently displayed menu level.
    pub fn current(&self) -> Option<&LeaderMenuDef<A>> {
        self.inner.current()
    }

    /// Breadcrumb trail for the title bar.
    pub fn breadcrumb(&self) -> &[String] {
        self.inner.breadcrumb()
    }

    /// Close the menu entirely.
    pub fn close(&mut self) {
        self.inner.close();
    }
}

impl<A: Clone> LeaderMenu<A> {
    /// Build from contributors (convenience wrapper).
    pub fn build(
        contributors: &[&dyn ratcore::leaderkey::MenuContributor<A>],
        hidden: &ratcore::leaderkey::HiddenSet,
    ) -> crate::builder::BuildResult<A> {
        crate::builder::build(contributors, hidden)
    }

    /// Open the menu (shows root level).
    pub fn open(&mut self) {
        self.inner.open();
    }

    /// Handle a crossterm key press while the menu is visible.
    ///
    /// Returns `Some(action)` if an action should be dispatched,
    /// `None` if the key was consumed internally (submenu nav, close).
    pub fn handle_key(&mut self, key: &KeyEvent) -> Option<LeaderAction<A>> {
        let input = crossterm_key_to_input(key);
        self.inner.handle_input(input)
    }
}

/// Convert a crossterm KeyEvent into a platform-neutral MenuInput.
fn crossterm_key_to_input(key: &KeyEvent) -> MenuInput {
    if key.code == KeyCode::Esc {
        return MenuInput::Escape;
    }

    match key.code {
        KeyCode::Char(c)
            if key.modifiers.is_empty() || key.modifiers == KeyModifiers::SHIFT =>
        {
            MenuInput::Char(c)
        }
        _ => MenuInput::Other,
    }
}

// ─────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

    use ratcore::leaderkey::{LeaderAction, LeaderMenuDef, LeaderMenuItem};
    use super::LeaderMenu;

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
        let core = ratcore::leaderkey::LeaderMenu::test_from_parts(root, submenus);
        LeaderMenu::from_core(core)
    }

    fn key(c: char) -> KeyEvent {
        KeyEvent::new_with_kind(KeyCode::Char(c), KeyModifiers::NONE, KeyEventKind::Press)
    }

    fn esc() -> KeyEvent {
        KeyEvent::new_with_kind(KeyCode::Esc, KeyModifiers::NONE, KeyEventKind::Press)
    }

    #[test]
    fn opens_and_closes() {
        let mut m = make_menu();
        assert!(!m.visible());
        m.open();
        assert!(m.visible());
        m.close();
        assert!(!m.visible());
    }

    #[test]
    fn esc_closes_root() {
        let mut m = make_menu();
        m.open();
        let r = m.handle_key(&esc());
        assert!(r.is_none());
        assert!(!m.visible());
    }

    #[test]
    fn direct_action_returns_and_closes() {
        let mut m = make_menu();
        m.open();
        let r = m.handle_key(&key('s'));
        assert_eq!(r, Some(LeaderAction::Action(Act::Save)));
        assert!(!m.visible());
    }

    #[test]
    fn submenu_navigation() {
        let mut m = make_menu();
        m.open();
        let r = m.handle_key(&key('x'));
        assert!(r.is_none());
        assert!(m.visible());

        let r = m.handle_key(&key('a'));
        assert_eq!(r, Some(LeaderAction::Command("/alpha".into())));
        assert!(!m.visible());
    }
}
