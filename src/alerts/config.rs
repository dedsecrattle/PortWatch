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
    pub fn canonical_path() -> Result<PathBuf> {
        let dir = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?;
        Ok(dir.join("portwatch").join("alerts.json"))
    }

    fn legacy_paths() -> Vec<PathBuf> {
        let mut paths = Vec::new();
        if let Some(home) = dirs::home_dir() {
            let xdg_style = home.join(".config").join("portwatch").join("alerts.json");
            paths.push(xdg_style);
        }
        paths
    }

    fn load_path_candidates() -> Result<Vec<PathBuf>> {
        let mut out = Vec::new();
        let canonical = Self::canonical_path()?;
        out.push(canonical.clone());

        for p in Self::legacy_paths() {
            if p != canonical && !out.contains(&p) {
                out.push(p);
            }
        }
        Ok(out)
    }

    pub fn load() -> Result<(Self, Option<PathBuf>)> {
        for path in Self::load_path_candidates()? {
            if path.exists() {
                let content = fs::read_to_string(&path)?;
                let config: AlertConfig = serde_json::from_str(&content)?;
                return Ok((config, Some(path)));
            }
        }
        Ok((Self::default(), None))
    }

    pub fn save(&self) -> Result<PathBuf> {
        let path = Self::canonical_path()?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        fs::write(&path, json)?;
        Ok(path)
    }

}

impl Default for AlertConfig {
    fn default() -> Self {
        Self { rules: Vec::new() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::alerts::{AlertCondition, AlertSeverity};

    fn rule(id: &str, name: &str, condition: AlertCondition) -> AlertRule {
        AlertRule {
            id: id.into(),
            name: name.into(),
            condition,
            enabled: true,
            severity: AlertSeverity::Warning,
            cooldown_seconds: 42,
        }
    }

    #[test]
    fn roundtrip_all_condition_variants() {
        let cfg = AlertConfig {
            rules: vec![
                rule("1", "p1", AlertCondition::PortOpened { port: 80 }),
                rule("2", "p2", AlertCondition::PortClosed { port: 443 }),
                rule(
                    "3",
                    "r",
                    AlertCondition::PortRangeActivity {
                        start_port: 1,
                        end_port: 1024,
                    },
                ),
                rule(
                    "4",
                    "ext",
                    AlertCondition::ExternalConnection {
                        ip_pattern: "^(10\\.)".into(),
                        exclude_private: false,
                    },
                ),
                rule(
                    "5",
                    "cpu",
                    AlertCondition::ProcessCpuThreshold {
                        process_pattern: "node".into(),
                        threshold_percent: 90.5,
                    },
                ),
                rule(
                    "6",
                    "mem",
                    AlertCondition::ProcessMemoryThreshold {
                        process_pattern: "java".into(),
                        threshold_mb: 1024,
                    },
                ),
                rule(
                    "7",
                    "unk",
                    AlertCondition::UnknownProcessListening,
                ),
            ],
        };
        let json = serde_json::to_string_pretty(&cfg).unwrap();
        let back: AlertConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(back.rules.len(), cfg.rules.len());
        assert!(matches!(
            back.rules[6].condition,
            AlertCondition::UnknownProcessListening
        ));
    }
}
