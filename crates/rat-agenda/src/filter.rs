//! Agenda filter logic.

use crate::types::AgendaItem;

/// A filter specification applied to agenda items.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct FilterSpec {
    /// Only show items with these statuses (empty = no filter).
    pub statuses: Vec<String>,
    /// Only show items with at least one of these tags.
    pub include_tags: Vec<String>,
    /// Exclude items with any of these tags.
    pub exclude_tags: Vec<String>,
    /// Only show items with these priorities (empty = no filter).
    pub priorities: Vec<char>,
    /// Free-text search on title (case-insensitive).
    pub text_query: String,
}

impl FilterSpec {
    pub fn is_empty(&self) -> bool {
        self.statuses.is_empty()
            && self.include_tags.is_empty()
            && self.exclude_tags.is_empty()
            && self.priorities.is_empty()
            && self.text_query.is_empty()
    }

    /// Check if an item passes all filters (AND logic).
    pub fn matches(&self, item: &AgendaItem) -> bool {
        // Status filter
        if !self.statuses.is_empty() {
            match &item.status {
                Some(s) if self.statuses.iter().any(|f| f == s) => {}
                _ => return false,
            }
        }
        // Tag include
        if !self.include_tags.is_empty() && !self.include_tags.iter().any(|t| item.tags.contains(t))
        {
            return false;
        }
        // Tag exclude
        if self.exclude_tags.iter().any(|t| item.tags.contains(t)) {
            return false;
        }
        // Priority
        if !self.priorities.is_empty() {
            match item.priority {
                Some(p) if self.priorities.contains(&p) => {}
                _ => return false,
            }
        }
        // Text search
        if !self.text_query.is_empty() {
            let q = self.text_query.to_lowercase();
            if !item.title.to_lowercase().contains(&q) {
                return false;
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{AgendaItem, Date};

    fn item(
        title: &str,
        status: Option<&str>,
        priority: Option<char>,
        tags: &[&str],
    ) -> AgendaItem {
        AgendaItem {
            id: title.into(),
            title: title.into(),
            status: status.map(|s| s.into()),
            priority,
            tags: tags.iter().map(|s| s.to_string()).collect(),
            scheduled: Some(Date::new(2026, 3, 15)),
            deadline: None,
            time_start: None,
            time_end: None,
            source_file: None,
            source_line: None,
        }
    }

    #[test]
    fn empty_filter_matches_all() {
        let f = FilterSpec::default();
        assert!(f.matches(&item("Task", None, None, &[])));
    }

    #[test]
    fn status_filter() {
        let f = FilterSpec {
            statuses: vec!["TODO".into()],
            ..Default::default()
        };
        assert!(f.matches(&item("A", Some("TODO"), None, &[])));
        assert!(!f.matches(&item("B", Some("DONE"), None, &[])));
        assert!(!f.matches(&item("C", None, None, &[])));
    }

    #[test]
    fn tag_include() {
        let f = FilterSpec {
            include_tags: vec!["work".into()],
            ..Default::default()
        };
        assert!(f.matches(&item("A", None, None, &["work"])));
        assert!(!f.matches(&item("B", None, None, &["personal"])));
    }

    #[test]
    fn tag_exclude() {
        let f = FilterSpec {
            exclude_tags: vec!["spam".into()],
            ..Default::default()
        };
        assert!(f.matches(&item("A", None, None, &["work"])));
        assert!(!f.matches(&item("B", None, None, &["spam"])));
    }

    #[test]
    fn text_search() {
        let f = FilterSpec {
            text_query: "ship".into(),
            ..Default::default()
        };
        assert!(f.matches(&item("Ship docs", None, None, &[])));
        assert!(!f.matches(&item("Buy groceries", None, None, &[])));
    }

    #[test]
    fn combined_and() {
        let f = FilterSpec {
            statuses: vec!["TODO".into()],
            priorities: vec!['A'],
            ..Default::default()
        };
        assert!(f.matches(&item("A", Some("TODO"), Some('A'), &[])));
        assert!(!f.matches(&item("B", Some("TODO"), Some('B'), &[])));
        assert!(!f.matches(&item("C", Some("DONE"), Some('A'), &[])));
    }
}
