pub mod entropy;
pub mod regex_guard;
pub mod binary_blocker;

use std::path::Path;
use std::fs;
use anyhow::Result;

pub struct Sentinel;

impl Sentinel {
    pub fn scan_file(path: &Path) -> Result<Vec<String>> {
        let mut issues = Vec::new();

        // 1. Binary Check (Simple extension/content check)
        if binary_blocker::is_binary(path.to_str().unwrap_or("")) {
             // For now, we just skip binary files or flag them if they are large
             // In a real scenario, we might want to block them if they are not tracked
             return Ok(vec!["Binary file detected".to_string()]);
        }

        let content = fs::read(path)?;

        // 2. Entropy Check
        let entropy = entropy::calculate_entropy(&content);
        if entropy > entropy::HIGH_ENTROPY_THRESHOLD {
            issues.push(format!("High entropy detected ({:.2}). Potential secret or encrypted data.", entropy));
        }

        // 3. Regex Guard
        // We need valid UTF-8 for regex
        if let Ok(text) = String::from_utf8(content) {
            if let Some(idx) = regex_guard::check_patterns(&text) {
                issues.push(format!("Secret pattern detected (pattern index: {}).", idx));
            }
        }

        Ok(issues)
    }
}

pub fn scan() {
    println!("Sentinel scanning...");
}
