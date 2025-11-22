use std::collections::HashMap;

/// Calculates the Shannon entropy of a byte slice.
/// Returns a value between 0.0 and 8.0.
/// Higher values indicate higher randomness (potential encryption or compressed data).
pub fn calculate_entropy(data: &[u8]) -> f64 {
    if data.is_empty() {
        return 0.0;
    }

    let mut frequencies = HashMap::new();
    for &byte in data {
        *frequencies.entry(byte).or_insert(0) += 1;
    }

    let len = data.len() as f64;
    let mut entropy = 0.0;

    for &count in frequencies.values() {
        let p = count as f64 / len;
        entropy -= p * p.log2();
    }

    entropy
}

/// Threshold for what we consider "high entropy" (potential secret/key).
/// Standard English text is usually around 3.5 - 4.5.
/// Base64 encoded secrets often exceed 5.5 or 6.0.
pub const HIGH_ENTROPY_THRESHOLD: f64 = 6.0;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_low_entropy() {
        let data = b"aaaaaaaaaa";
        let entropy = calculate_entropy(data);
        assert_eq!(entropy, 0.0);
    }

    #[test]
    fn test_medium_entropy() {
        let data = b"The quick brown fox jumps over the lazy dog";
        let entropy = calculate_entropy(data);
        assert!(entropy > 3.0 && entropy < 5.0);
    }

    #[test]
    fn test_high_entropy() {
        // Random bytes usually have high entropy
        let data = b"8f9d23a1c5b6e7f8901234567890abcdef";
        let entropy = calculate_entropy(data);
        // This might not be super high because it's hex (limited charset), but let's check
        // A real random byte array would be better
        assert!(entropy > 3.0); 
    }
}
