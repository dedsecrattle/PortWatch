use super::{AlertCondition, AlertRule, AlertSeverity};
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
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?;
        Ok(config_dir.join("portwatch").join("alerts.json"))
    }

    pub fn default_rules() -> Vec<AlertRule> {
        vec![
            AlertRule::new(
                "privileged-port".to_string(),
                "Privileged Port Opened".to_string(),
                AlertCondition::PortRangeActivity {
                    start_port: 1,
                    end_port: 1024,
                },
                AlertSeverity::Warning,
            ),
            AlertRule::new(
                "external-connection".to_string(),
                "External Connection Detected".to_string(),
                AlertCondition::ExternalConnection {
                    ip_pattern: ".*".to_string(),
                    exclude_private: true,
                },
                AlertSeverity::Info,
            ),
        ]
    }
}

impl Default for AlertConfig {
    fn default() -> Self {
        Self {
            rules: Self::default_rules(),
        }
    }
}
