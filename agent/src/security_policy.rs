use serde::{Deserialize, Serialize};

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