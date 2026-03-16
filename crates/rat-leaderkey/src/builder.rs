//! Menu building: collect contributions, resolve conflicts, assemble tree.

use std::collections::HashMap;

use crate::registry::Conflict;
use crate::state::LeaderMenu;
use crate::types::*;

/// Result of building a leader menu: the menu and any conflicts detected.
pub type BuildResult<A> = (LeaderMenu<A>, Vec<Conflict>);

/// Build a leader menu from contributors.
///
/// Collects all [`MenuContribution`] items, deduplicates by `(key, placement)`
/// with highest priority winning, removes hidden entries, and assembles the
/// menu tree.
pub fn build<A: Clone>(
    contributors: &[&dyn MenuContributor<A>],
    hidden: &HiddenSet,
) -> BuildResult<A> {
    let items = contributors.iter().flat_map(|c| c.menu_items()).collect();
    build_from_items(items, hidden)
}

/// Build a leader menu from a pre-collected list of contributions.
///
/// Same conflict resolution and assembly as [`build`], but takes items directly
/// instead of going through the [`MenuContributor`] trait. Useful when the
/// caller already has a flat `Vec<MenuContribution>`.
pub fn build_from_items<A: Clone>(
    mut all_items: Vec<MenuContribution<A>>,
    hidden: &HiddenSet,
) -> BuildResult<A> {
    let mut conflicts = Vec::new();

    // 1. Sort by priority (lowest first so highest overwrites).
    all_items.sort_by_key(|i| i.priority);

    // 3. Deduplicate by (key, placement) — last writer wins.
    let mut seen: HashMap<(char, MenuPlacement), MenuContribution<A>> = HashMap::new();
    for item in all_items {
        let slot = (item.key, item.placement.clone());
        if let Some(existing) = seen.get(&slot) {
            conflicts.push(Conflict {
                registry: "leader_menu",
                key: format!("'{}' in {:?}", item.key, item.placement),
                winner: item.source.clone(),
                loser: existing.source.clone(),
            });
        }
        seen.insert(slot, item);
    }

    // 4. Remove hidden entries.
    for h in hidden {
        seen.remove(h);
    }

    // 5. Group by placement.
    let mut root_items: Vec<MenuContribution<A>> = Vec::new();
    let mut submenu_items: HashMap<String, Vec<MenuContribution<A>>> = HashMap::new();

    for ((_, placement), item) in seen {
        match placement {
            MenuPlacement::Root => root_items.push(item),
            MenuPlacement::Submenu(ref name) => {
                submenu_items.entry(name.clone()).or_default().push(item);
            }
        }
    }

    // 6. Build submenu defs (sorted by key for stable ordering).
    let mut submenus: Vec<LeaderMenuDef<A>> = Vec::new();
    for (name, mut items) in submenu_items {
        items.sort_by_key(|i| i.key);
        submenus.push(LeaderMenuDef {
            label: name,
            items: items
                .into_iter()
                .map(|c| LeaderMenuItem {
                    key: c.key,
                    label: c.label,
                    action: c.action,
                })
                .collect(),
        });
    }

    // 7. Build root def (sorted by key).
    root_items.sort_by_key(|i| i.key);
    let root = LeaderMenuDef {
        label: "Leader".into(),
        items: root_items
            .into_iter()
            .map(|c| LeaderMenuItem {
                key: c.key,
                label: c.label,
                action: c.action,
            })
            .collect(),
    };

    let menu = LeaderMenu::from_parts(root, submenus);
    (menu, conflicts)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;
    use crate::{PRIORITY_BUILTIN, PRIORITY_PLUGIN};

    #[derive(Debug, Clone, PartialEq, Eq)]
    enum Act {
        Save,
        Open,
    }

    struct TestContributor {
        items: Vec<MenuContribution<Act>>,
    }

    impl MenuContributor<Act> for TestContributor {
        fn menu_items(&self) -> Vec<MenuContribution<Act>> {
            self.items.clone()
        }
    }

    #[test]
    fn single_contributor() {
        let c = TestContributor {
            items: vec![
                MenuContribution {
                    key: 'a',
                    label: "alpha".into(),
                    action: LeaderAction::Command("/alpha".into()),
                    placement: MenuPlacement::Root,
                    priority: PRIORITY_BUILTIN,
                    source: "test".into(),
                },
                MenuContribution {
                    key: 'b',
                    label: "beta".into(),
                    action: LeaderAction::Command("/beta".into()),
                    placement: MenuPlacement::Root,
                    priority: PRIORITY_BUILTIN,
                    source: "test".into(),
                },
            ],
        };

        let (menu, conflicts) = build(&[&c], &HashSet::new());
        assert!(conflicts.is_empty());
        assert_eq!(menu.root_def().items.len(), 2);
        assert_eq!(menu.root_def().items[0].key, 'a');
        assert_eq!(menu.root_def().items[1].key, 'b');
    }

    #[test]
    fn higher_priority_wins() {
        let lo = TestContributor {
            items: vec![MenuContribution {
                key: 'x',
                label: "lo".into(),
                action: LeaderAction::Action(Act::Save),
                placement: MenuPlacement::Root,
                priority: PRIORITY_BUILTIN,
                source: "builtin".into(),
            }],
        };
        let hi = TestContributor {
            items: vec![MenuContribution {
                key: 'x',
                label: "hi".into(),
                action: LeaderAction::Action(Act::Open),
                placement: MenuPlacement::Root,
                priority: PRIORITY_PLUGIN,
                source: "plugin".into(),
            }],
        };

        let (menu, conflicts) = build(&[&lo, &hi], &HashSet::new());
        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0].winner, "plugin");
        assert_eq!(conflicts[0].loser, "builtin");
        assert_eq!(menu.root_def().items[0].label, "hi");
    }

    #[test]
    fn user_overrides_everything() {
        let lo = TestContributor {
            items: vec![MenuContribution {
                key: 'z',
                label: "builtin-z".into(),
                action: LeaderAction::Action(Act::Save),
                placement: MenuPlacement::Root,
                priority: PRIORITY_BUILTIN,
                source: "builtin".into(),
            }],
        };
        let hi = TestContributor {
            items: vec![MenuContribution {
                key: 'z',
                label: "user-z".into(),
                action: LeaderAction::Action(Act::Open),
                placement: MenuPlacement::Root,
                priority: crate::PRIORITY_USER,
                source: "config".into(),
            }],
        };

        let (menu, _) = build(&[&lo, &hi], &HashSet::new());
        assert_eq!(menu.root_def().items[0].label, "user-z");
    }

    #[test]
    fn hidden_excluded() {
        let c = TestContributor {
            items: vec![
                MenuContribution {
                    key: 'a',
                    label: "keep".into(),
                    action: LeaderAction::Action(Act::Save),
                    placement: MenuPlacement::Root,
                    priority: PRIORITY_BUILTIN,
                    source: "test".into(),
                },
                MenuContribution {
                    key: 'b',
                    label: "hide".into(),
                    action: LeaderAction::Action(Act::Open),
                    placement: MenuPlacement::Root,
                    priority: PRIORITY_BUILTIN,
                    source: "test".into(),
                },
            ],
        };

        let mut hidden = HashSet::new();
        hidden.insert(('b', MenuPlacement::Root));

        let (menu, _) = build(&[&c], &hidden);
        assert_eq!(menu.root_def().items.len(), 1);
        assert_eq!(menu.root_def().items[0].key, 'a');
    }

    #[test]
    fn submenu_auto_creation() {
        let c = TestContributor {
            items: vec![
                MenuContribution {
                    key: 'p',
                    label: "plugins".into(),
                    action: LeaderAction::Submenu("plugins".into()),
                    placement: MenuPlacement::Root,
                    priority: PRIORITY_BUILTIN,
                    source: "test".into(),
                },
                MenuContribution {
                    key: 'c',
                    label: "calendar".into(),
                    action: LeaderAction::Command("/cal".into()),
                    placement: MenuPlacement::Submenu("plugins".into()),
                    priority: PRIORITY_PLUGIN,
                    source: "calendar".into(),
                },
            ],
        };

        let (menu, _) = build(&[&c], &HashSet::new());
        assert_eq!(menu.root_def().items.len(), 1);
        assert_eq!(menu.root_def().items[0].key, 'p');

        let subs = menu.submenu_defs();
        let plugins = subs.iter().find(|s| s.label == "plugins").unwrap();
        assert_eq!(plugins.items.len(), 1);
        assert_eq!(plugins.items[0].key, 'c');
    }

    #[test]
    fn empty_build() {
        let (menu, conflicts) = build::<Act>(&[], &HashSet::new());
        assert!(conflicts.is_empty());
        assert!(menu.root_def().items.is_empty());
        assert!(menu.submenu_defs().is_empty());
    }

    #[test]
    fn same_key_different_placement_no_conflict() {
        let c = TestContributor {
            items: vec![
                MenuContribution {
                    key: 'a',
                    label: "root-a".into(),
                    action: LeaderAction::Action(Act::Save),
                    placement: MenuPlacement::Root,
                    priority: PRIORITY_BUILTIN,
                    source: "test".into(),
                },
                MenuContribution {
                    key: 'a',
                    label: "sub-a".into(),
                    action: LeaderAction::Action(Act::Open),
                    placement: MenuPlacement::Submenu("foo".into()),
                    priority: PRIORITY_BUILTIN,
                    source: "test".into(),
                },
            ],
        };

        let (_, conflicts) = build(&[&c], &HashSet::new());
        assert!(conflicts.is_empty());
    }
}
