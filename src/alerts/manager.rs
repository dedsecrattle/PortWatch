use std::collections::HashMap;
use chrono::{DateTime, Utc};
use super::{Alert, AlertRule};

pub struct AlertManager {
    rules: Vec<AlertRule>,
    last_triggered: HashMap<String, DateTime<Utc>>,
    alert_history: Vec<Alert>,
    max_history: usize,
}

impl AlertManager {
    pub fn new() -> Self {
        Self {
            rules: Vec::new(),
            last_triggered: HashMap::new(),
            alert_history: Vec::new(),
            max_history: 100,
        }
    }

    pub fn add_rule(&mut self, rule: AlertRule) {
        self.rules.push(rule);
    }

    pub fn get_enabled_rules(&self) -> Vec<&AlertRule> {
        self.rules.iter().filter(|r| r.enabled).collect()
    }

    pub fn can_trigger(&self, rule_id: &str, cooldown_seconds: u64) -> bool {
        if let Some(last_time) = self.last_triggered.get(rule_id) {
            let elapsed = Utc::now().signed_duration_since(*last_time);
            elapsed.num_seconds() as u64 >= cooldown_seconds
        } else {
            true
        }
    }

    pub fn trigger_alert(&mut self, alert: Alert) {
        self.last_triggered.insert(alert.rule_id.clone(), alert.timestamp);
        self.alert_history.push(alert);
        
        if self.alert_history.len() > self.max_history {
            self.alert_history.remove(0);
        }
    }

    pub fn get_recent_alerts(&self, count: usize) -> &[Alert] {
        let start = self.alert_history.len().saturating_sub(count);
        &self.alert_history[start..]
    }
}

impl Default for AlertManager {
    fn default() -> Self {
        Self::new()
    }
}
