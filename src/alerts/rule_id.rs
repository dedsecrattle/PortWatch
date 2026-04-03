use super::AlertCondition;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Stable id derived from rule name + condition (for cooldown keys). Changes if semantics change.
pub fn derive_rule_id(name: &str, condition: &AlertCondition) -> String {
    let mut hasher = DefaultHasher::new();
    name.hash(&mut hasher);
    let cond_json = serde_json::to_string(condition).unwrap_or_default();
    cond_json.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}
