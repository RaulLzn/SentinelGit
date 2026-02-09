use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    pub sentinel: SentinelConfig,
    pub chronos: ChronosConfig,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SentinelConfig {
    pub secret_patterns: Vec<String>,
    pub binary_extensions: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ChronosConfig {
    pub enabled: bool,
    pub db_path: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            sentinel: SentinelConfig {
                secret_patterns: vec![
                    r"(?i)aws_access_key_id\s*=\s*[A-Z0-9]{20}".to_string(),
                    r"(?i)aws_secret_access_key\s*=\s*[A-Za-z0-9/+=]{40}".to_string(),
                    r"(?i)private_key\s*=\s*-----BEGIN RSA PRIVATE KEY-----".to_string(),
                    r"(?i)api_key\s*=\s*[A-Za-z0-9]{32,}".to_string(),
                ],
                binary_extensions: vec![
                    "exe", "dll", "so", "dylib", "o", "obj", "zip", "tar", "gz", "7z", "rar",
                    "jpg", "jpeg", "png", "gif", "bmp", "ico", "pdf", "doc", "docx", "xls", "xlsx",
                    "ppt", "pptx", "mp3", "mp4", "avi", "mov", "flv", "wmv", "class", "jar", "war",
                    "ear", "pyc", "pyd",
                ]
                .into_iter()
                .map(|s| s.to_string())
                .collect(),
            },
            chronos: ChronosConfig {
                enabled: true,
                db_path: None,
            },
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        let mut config = Config::default();

        // 1. Global Config (~/.config/sgit/config.toml)
        if let Some(proj_dirs) = dirs::config_dir() {
            let global_path = proj_dirs.join("sgit/config.toml");
            if global_path.exists() {
                if let Ok(content) = fs::read_to_string(&global_path) {
                    if let Ok(global_config) = toml::from_str::<Config>(&content) {
                        config = global_config; // Override defaults
                    }
                }
            }
        }

        // 2. Local Config (.sgit.toml)
        let local_path = Path::new(".sgit.toml");
        if local_path.exists() {
            if let Ok(content) = fs::read_to_string(local_path) {
                // Determine if we are loading a full config or just partial overrides.
                // For now, let's assume if it parses as Config we take it.
                // A more robust implementation would use something like `config-rs` crate,
                // but we want to keep dependencies minimal if possible or just use what we have.
                // Re-implementing a manual merge:

                #[derive(Deserialize)]
                struct PartialConfig {
                    sentinel: Option<PartialSentinelConfig>,
                    chronos: Option<PartialChronosConfig>,
                }
                #[derive(Deserialize)]
                struct PartialSentinelConfig {
                    secret_patterns: Option<Vec<String>>,
                    binary_extensions: Option<Vec<String>>,
                }
                #[derive(Deserialize)]
                struct PartialChronosConfig {
                    enabled: Option<bool>,
                    db_path: Option<String>,
                }

                if let Ok(partial) = toml::from_str::<PartialConfig>(&content) {
                    if let Some(s) = partial.sentinel {
                        if let Some(patterns) = s.secret_patterns {
                            config.sentinel.secret_patterns.extend(patterns);
                            config.sentinel.secret_patterns.sort();
                            config.sentinel.secret_patterns.dedup();
                        }
                        if let Some(exts) = s.binary_extensions {
                            config.sentinel.binary_extensions.extend(exts);
                            config.sentinel.binary_extensions.sort();
                            config.sentinel.binary_extensions.dedup();
                        }
                    }
                    if let Some(c) = partial.chronos {
                        if let Some(enabled) = c.enabled {
                            config.chronos.enabled = enabled;
                        }
                        if let Some(path) = c.db_path {
                            config.chronos.db_path = Some(path);
                        }
                    }
                }
            }
        }

        Ok(config)
    }
}
