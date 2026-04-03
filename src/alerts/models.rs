use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "params")]
pub enum AlertCondition {
    PortOpened { port: u16 },
    PortClosed { port: u16 },
    ExternalConnection { 
        ip_pattern: String,
        exclude_private: bool,
    },
    ProcessCpuThreshold { 
        process_pattern: String,
        threshold_percent: f32,
    },
    ProcessMemoryThreshold { 
        process_pattern: String,
        threshold_mb: u64,
    },
    UnknownProcessListening,
    PortRangeActivity { 
        start_port: u16, 
        end_port: u16,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertRule {
    pub id: String,
    pub name: String,
    pub condition: AlertCondition,
    pub enabled: bool,
    pub severity: AlertSeverity,
    pub cooldown_seconds: u64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
}


#[derive(Debug, Clone)]
pub struct Alert {
    pub rule_id: String,
    pub message: String,
    pub severity: AlertSeverity,
    pub timestamp: DateTime<Utc>,
}

impl Alert {
    pub fn new(
        rule_id: String,
        message: String,
        severity: AlertSeverity,
    ) -> Self {
        Self {
            rule_id,
            message,
            severity,
            timestamp: Utc::now(),
        }
    }
}
