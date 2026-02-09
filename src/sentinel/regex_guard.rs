use regex::RegexSet;

pub fn compile_patterns(patterns: &[String]) -> Option<RegexSet> {
    RegexSet::new(patterns).ok()
}

pub fn check_patterns(text: &str, set: &RegexSet) -> Option<usize> {
    let matches = set.matches(text);
    if matches.matched_any() {
        // Return the index of the first matched pattern
        matches.iter().next()
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aws_key_detection() {
        let patterns = vec![r"(?i)aws_access_key_id\s*=\s*[A-Z0-9]{20}".to_string()];
        let set = compile_patterns(&patterns).unwrap();
        let text = "aws_access_key_id = AKIAIOSFODNN7EXAMPLE";
        assert!(check_patterns(text, &set).is_some());
    }

    #[test]
    fn test_safe_text() {
        let patterns = vec![r"(?i)aws_access_key_id\s*=\s*[A-Z0-9]{20}".to_string()];
        let set = compile_patterns(&patterns).unwrap();
        let text = "This is a safe configuration file.";
        assert!(check_patterns(text, &set).is_none());
    }
}
