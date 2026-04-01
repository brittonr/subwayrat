//! Menu building — delegates to ratcore, wraps result in platform LeaderMenu.

use ratcore::leaderkey::{self, Conflict, HiddenSet, MenuContribution, MenuContributor};

use crate::state::LeaderMenu;

/// Result of building a leader menu: the menu and any conflicts detected.
pub type BuildResult<A> = (LeaderMenu<A>, Vec<Conflict>);

/// Build a leader menu from contributors.
pub fn build<A: Clone>(
    contributors: &[&dyn MenuContributor<A>],
    hidden: &HiddenSet,
) -> BuildResult<A> {
    let (core_menu, conflicts) = leaderkey::build(contributors, hidden);
    (LeaderMenu::from_core(core_menu), conflicts)
}

/// Build from a pre-collected list of contributions.
pub fn build_from_items<A: Clone>(
    items: Vec<MenuContribution<A>>,
    hidden: &HiddenSet,
) -> BuildResult<A> {
    let (core_menu, conflicts) = leaderkey::build_from_items(items, hidden);
    (LeaderMenu::from_core(core_menu), conflicts)
}
