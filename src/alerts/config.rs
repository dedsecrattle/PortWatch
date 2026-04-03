use super::AlertRule;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct AlertConfig {
    pub rules: Vec<AlertRule>,
}

impl AlertConfig {
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;
        
        if !config_path.exists() {
            return Ok(Self::default());
        }
        
        let content = fs::read_to_string(config_path)?;
        let config: AlertConfig = serde_json::from_str(&content)?;
        Ok(config)
    }

    fn config_path() -> Result<PathBuf> {
        // Prefer the traditional XDG-like path on all platforms.
        // On macOS this intentionally differs from `dirs::config_dir()` (which is
        // typically `~/Library/Application Support`).
        let home_dir = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
        let preferred = home_dir.join(".config").join("portwatch").join("alerts.json");

        // Backwards-compatible fallback: if the old location exists, use it.
        let fallback = dirs::config_dir()
            .map(|d| d.join("portwatch").join("alerts.json"));

        if preferred.exists() {
            return Ok(preferred);
        }

        if let Some(fallback) = fallback {
            if fallback.exists() {
                return Ok(fallback);
            }
        }

        Ok(preferred)
    }
}

impl Default for AlertConfig {
    fn default() -> Self {
        Self { rules: Vec::new() }
    }
}
