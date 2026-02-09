pub mod binary_blocker;
pub mod entropy;
pub mod regex_guard;

use crate::config::Config;
use anyhow::Result;
use regex::RegexSet;
use std::fs::File;
use std::io::Read;
use std::path::Path;

pub struct Sentinel {
    binary_extensions: Vec<String>,
    regex_set: Option<RegexSet>,
}

impl Sentinel {
    pub fn new(config: &Config) -> Self {
        let regex_set = regex_guard::compile_patterns(&config.sentinel.secret_patterns);
        Self {
            binary_extensions: config.sentinel.binary_extensions.clone(),
            regex_set,
        }
    }

    pub fn scan_file(&self, path: &Path) -> Result<Vec<String>> {
        let mut issues = Vec::new();

        // 1. Binary Check (Simple extension/content check)
        if binary_blocker::is_binary(path.to_str().unwrap_or(""), &self.binary_extensions) {
            return Ok(vec!["Binary file detected".to_string()]);
        }

        // Open file once
        let mut file = File::open(path)?;
        let metadata = file.metadata()?;
        let len = metadata.len();

        // 2. Content Checks
        // If file is too large (> 10MB), skip detailed scans or handle differently
        if len > 10 * 1024 * 1024 {
            return Ok(vec!["File too large for content scan".to_string()]);
        }

        let mut content = Vec::new();
        file.read_to_end(&mut content)?;

        // 2a. Content-based binary check (null bytes)
        // binary_blocker::is_binary does checking on filename AND null bytes reading file again.
        // We should probably separate them or let it be.
        // For now, let's just rely on entropy and regex since we already read it.
        // NOTE: The previous `is_binary` implementation did both. We updated it to take extensions.
        // Let's assume `binary_blocker::is_binary` still does the content check if we didn't change the second part.
        // Wait, I only replaced the first part of `is_binary` in the previous tool call.
        // The second part (null byte check) re-opens the file. That's inefficient if we strictly want to stream,
        // but for now it's okay.

        // 3. Entropy Check
        let entropy = entropy::calculate_entropy(&content);
        if entropy > entropy::HIGH_ENTROPY_THRESHOLD {
            issues.push(format!(
                "High entropy detected ({:.2}). Potential secret or encrypted data.",
                entropy
            ));
        }

        // 4. Regex Guard
        // We need valid UTF-8 for regex
        if let Some(set) = &self.regex_set {
            if let Ok(text) = String::from_utf8(content) {
                if let Some(idx) = regex_guard::check_patterns(&text, set) {
                    issues.push(format!("Secret pattern detected (pattern index: {}).", idx));
                }
            }
        }

        Ok(issues)
    }
}
