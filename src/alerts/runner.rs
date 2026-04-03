use std::collections::HashMap;

use crate::models::PortRecord;

use super::{AlertEvaluator, AlertManager, Notifier};

pub struct TriggeredAlert {
    pub message: String,
    pub notify_result: anyhow::Result<()>,
}

/// Evaluate enabled rules against the previous and current port snapshots and send notifications.
/// Updates `alert_manager` state (cooldown / history) for any fired alerts.
pub fn run_alert_cycle(
    alert_manager: &mut AlertManager,
    notifier: &Notifier,
    previous_ports: &[PortRecord],
    current_ports: &[PortRecord],
    cpu_usage: &HashMap<u32, f32>,
    memory_usage: &HashMap<u32, u64>,
) -> Vec<TriggeredAlert> {
    let enabled_rules: Vec<_> = alert_manager
        .get_enabled_rules()
        .into_iter()
        .cloned()
        .collect();

    let mut out = Vec::new();

    for rule in enabled_rules {
        if !alert_manager.can_trigger(&rule.id, rule.cooldown_seconds) {
            continue;
        }

        let alert = match &rule.condition {
            crate::alerts::AlertCondition::PortOpened { .. }
            | crate::alerts::AlertCondition::PortClosed { .. }
            | crate::alerts::AlertCondition::PortRangeActivity { .. } => {
                AlertEvaluator::evaluate_port_changes(&rule, previous_ports, current_ports)
            }
            crate::alerts::AlertCondition::ExternalConnection { .. } => {
                AlertEvaluator::evaluate_connections(&rule, current_ports)
            }
            crate::alerts::AlertCondition::ProcessCpuThreshold { .. } => {
                AlertEvaluator::evaluate_process_cpu(&rule, current_ports, cpu_usage)
            }
            crate::alerts::AlertCondition::ProcessMemoryThreshold { .. } => {
                AlertEvaluator::evaluate_process_memory(&rule, current_ports, memory_usage)
            }
            _ => None,
        };

        if let Some(alert) = alert {
            let message = alert.message.clone();
            let notify_result = notifier.send(&alert);
            alert_manager.trigger_alert(alert);
            out.push(TriggeredAlert {
                message,
                notify_result,
            });
        }
    }

    out
}
