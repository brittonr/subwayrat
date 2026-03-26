//! Tag data model: validation, parse, format.

/// Format tags as `:tag1:tag2:` string. Empty vec → empty string.
pub fn format_tags(tags: &[String]) -> String {
    if tags.is_empty() { String::new() }
    else { format!(":{}:", tags.join(":")) }
}

/// Parse `:tag1:tag2:` string into a Vec.
pub fn parse_tags(s: &str) -> Vec<String> {
    s.split(':').filter(|t| !t.is_empty()).map(|t| t.to_string()).collect()
}

/// Check if a tag string is valid (alphanumeric + underscore + hyphen).
pub fn is_valid_tag(tag: &str) -> bool {
    !tag.is_empty() && tag.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_roundtrip() {
        let tags = vec!["work".into(), "urgent".into()];
        let s = format_tags(&tags);
        assert_eq!(s, ":work:urgent:");
        assert_eq!(parse_tags(&s), tags);
    }

    #[test]
    fn format_empty() {
        assert_eq!(format_tags(&[]), "");
    }

    #[test]
    fn valid_tags() {
        assert!(is_valid_tag("work"));
        assert!(is_valid_tag("my-tag"));
        assert!(is_valid_tag("tag_123"));
        assert!(!is_valid_tag(""));
        assert!(!is_valid_tag("has space"));
        assert!(!is_valid_tag("has:colon"));
    }
}
