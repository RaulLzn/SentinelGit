use regex::RegexSet;
use lazy_static::lazy_static;

lazy_static! {
    static ref SECRET_PATTERNS: RegexSet = RegexSet::new(&[
        r"(?i)aws_access_key_id\s*=\s*[A-Z0-9]{20}",
        r"(?i)aws_secret_access_key\s*=\s*[A-Za-z0-9/+=]{40}",
        r"(?i)private_key\s*=\s*-----BEGIN RSA PRIVATE KEY-----",
        r"(?i)api_key\s*=\s*[A-Za-z0-9]{32,}",
        // Add more patterns here
    ]).unwrap();
}

pub fn check_patterns(text: &str) -> Option<usize> {
    let matches = SECRET_PATTERNS.matches(text);
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
        let text = "aws_access_key_id = AKIAIOSFODNN7EXAMPLE";
        assert!(check_patterns(text).is_some());
    }

    #[test]
    fn test_safe_text() {
        let text = "This is a safe configuration file.";
        assert!(check_patterns(text).is_none());
    }
}
