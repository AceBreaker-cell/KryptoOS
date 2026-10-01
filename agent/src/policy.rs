use serde::{Deserialize, Serialize};
use sled::Db;
use std::sync::Arc;
use tokio::sync::Mutex;
use chrono::{DateTime, Utc};

/// Represents a security rule in the KryptonOS policy engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityRule {
    pub id: String,
    pub rule_type: RuleType,
    pub action: RuleAction,
    pub criteria: RuleCriteria,
    pub enabled: bool,
    pub description: String,
}

/// Types of rules that can be enforced
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuleType {
    Network,    // Socket connections, DNS, etc.
    Process,    // Process execution, forks, etc.
    Device,     // USB mounting, device access, etc.
    File,       // File access, modifications, etc.
}

/// Actions to take when a rule matches
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuleAction {
    Allow,
    Deny,
    Prompt,     // Show interactive prompt to user
    LogOnly,    // Just log the event
}

/// Criteria for matching rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleCriteria {
    pub hash: Option<String>,           // Hash of binary/file
    pub path: Option<String>,           // File path or socket path
    pub port: Option<u16>,              // Network port
    pub protocol: Option<String>,       // TCP/UDP/etc.
    pub process_name: Option<String>,   // Name of process
    pub device_id: Option<String>,      // USB device ID
    pub user_id: Option<String>,        // User executing action
    pub custom: Option<serde_json::Value>, // Custom criteria
}

/// Policy engine for evaluating security rules
pub struct PolicyEngine {
    db: Arc<Mutex<sled::Db>>,
}

impl PolicyEngine {
    /// Create a new policy engine instance
    pub fn new(db: Arc<Mutex<Db>>) -> Self {
        PolicyEngine { db }
    }

    /// Initialize the database with default rules if empty
    pub async fn initialize_default_rules(&self) -> Result<(), sled::Error> {
        let db = self.db.lock().await;

        // Check if we already have rules
        if db.contains_key(b"rules::count")? {
            return Ok(());
        }

        // Insert some default deny-all rules for demonstration
        let default_rules = vec![
            SecurityRule {
                id: "default-network-deny".to_string(),
                rule_type: RuleType::Network,
                action: RuleAction::Deny,
                criteria: RuleCriteria {
                    hash: None,
                    path: None,
                    port: None,
                    protocol: None,
                    process_name: None,
                    device_id: None,
                    user_id: None,
                    custom: None,
                },
                enabled: true,
                description: "Default deny all network connections".to_string(),
            },
            SecurityRule {
                id: "default-process-deny".to_string(),
                rule_type: RuleType::Process,
                action: RuleAction::Deny,
                criteria: RuleCriteria {
                    hash: None,
                    path: None,
                    port: None,
                    protocol: None,
                    process_name: None,
                    device_id: None,
                    user_id: None,
                    custom: None,
                },
                enabled: true,
                description: "Default deny all process executions".to_string(),
            }
        ];

        // Store rules in DB
        for (i, rule) in default_rules.iter().enumerate() {
            let key = format!("rule::{}", i).into_bytes();
            let value = serde_json::to_vec(rule).unwrap();
            db.insert(key, value)?;
        }

        // Store count
        db.insert(b"rules::count", default_rules.len().to_string().as_bytes())?;

        Ok(())
    }

    /// Evaluate an event against the policy rules
    pub async fn evaluate_event(&self, event: &SecurityEvent) -> RuleAction {
        let db = self.db.lock().await;

        // Get rule count
        let count_key = b"rules::count";
        let count = match db.get(count_key) {
            Ok(Some(val)) => String::from_utf8_lossy(&val).parse::<usize>().unwrap_or(0),
            Ok(None) => 0,
            Err(_) => 0,
        };

        // Check each rule
        for i in 0..count {
            let key = format!("rule::{}", i).into_bytes();
            if let Ok(Some(val)) = db.get(&key) {
                if let Ok(rule) = serde_json::from_slice::<SecurityRule>(&val) {
                    if rule.enabled && self.rule_matches(&rule, event) {
                        return rule.action.clone();
                    }
                }
            }
        }

        // Default to deny if no matching rule found
        RuleAction::Deny
    }

    /// Check if a rule matches a given event
    fn rule_matches(&self, rule: &SecurityRule, event: &SecurityEvent) -> bool {
        // Type must match
        if rule.rule_type != event.rule_type {
            return false;
        }

        // Check each criteria field
        if let Some(ref hash) = rule.criteria.hash {
            if let Some(ref event_hash) = event.hash {
                if hash != event_hash {
                    return false;
                }
            } else {
                return false; // Rule requires hash but event doesn't have one
            }
        }

        if let Some(ref path) = rule.criteria.path {
            if let Some(ref event_path) = event.path {
                if path != event_path {
                    return false;
                }
            } else {
                return false; // Rule requires path but event doesn't have one
            }
        }

        if let Some(port) = rule.criteria.port {
            if let Some(event_port) = event.port {
                if port != event_port {
                    return false;
                }
            } else {
                return false; // Rule requires port but event doesn't have one
            }
        }

        if let Some(ref protocol) = rule.criteria.protocol {
            if let Some(ref event_protocol) = event.protocol {
                if protocol != event_protocol {
                    return false;
                }
            } else {
                return false; // Rule requires protocol but event doesn't have one
            }
        }

        if let Some(ref process_name) = rule.criteria.process_name {
            if let Some(ref event_process_name) = event.process_name {
                if process_name != event_process_name {
                    return false;
                }
            } else {
                return false; // Rule requires process name but event doesn't have one
            }
        }

        if let Some(ref device_id) = rule.criteria.device_id {
            if let Some(ref event_device_id) = event.device_id {
                if device_id != event_device_id {
                    return false;
                }
            } else {
                return false; // Rule requires device ID but event doesn't have one
            }
        }

        // All criteria matched
        true
    }
}

/// Represents a security event that needs to be evaluated
#[derive(Debug, Clone)]
pub struct SecurityEvent {
    pub rule_type: RuleType,
    pub hash: Option<String>,
    pub path: Option<String>,
    pub port: Option<u16>,
    pub protocol: Option<String>,
    pub process_name: Option<String>,
    pub device_id: Option<String>,
    pub user_id: Option<String>,
    pub custom: Option<serde_json::Value>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl SecurityEvent {
    /// Create a new security event
    pub fn new(rule_type: RuleType) -> Self {
        SecurityEvent {
            rule_type,
            hash: None,
            path: None,
            port: None,
            protocol: None,
            process_name: None,
            device_id: None,
            user_id: None,
            custom: None,
            timestamp: chrono::Utc::now(),
        }
    }
}