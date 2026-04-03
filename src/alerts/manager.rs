use chrono::{DateTime, Utc};
use std::collections::HashMap;

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

    pub fn set_rules(&mut self, rules: Vec<AlertRule>) {
        self.rules = rules;
    }

    pub fn get_rules(&self) -> &[AlertRule] {
        &self.rules
    }

    pub fn add_rule(&mut self, rule: AlertRule) {
        self.rules.push(rule);
    }

    pub fn update_rule(&mut self, index: usize, new_rule: AlertRule) -> anyhow::Result<()> {
        anyhow::ensure!(
            index < self.rules.len(),
            "rule index out of range"
        );
        let old_id = self.rules[index].id.clone();
        if old_id != new_rule.id {
            self.last_triggered.remove(&old_id);
        }
        self.rules[index] = new_rule;
        Ok(())
    }

    pub fn delete_rule(&mut self, index: usize) -> anyhow::Result<AlertRule> {
        anyhow::ensure!(
            index < self.rules.len(),
            "rule index out of range"
        );
        let removed = self.rules.remove(index);
        self.last_triggered.remove(&removed.id);
        Ok(removed)
    }

    pub fn toggle_rule_enabled(&mut self, index: usize) -> anyhow::Result<bool> {
        anyhow::ensure!(
            index < self.rules.len(),
            "rule index out of range"
        );
        self.rules[index].enabled = !self.rules[index].enabled;
        Ok(self.rules[index].enabled)
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
