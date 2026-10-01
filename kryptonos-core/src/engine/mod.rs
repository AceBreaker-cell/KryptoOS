use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use chrono::{Utc, DateTime};
use serde::{Deserialize, Serialize};
use sled::Db;
use tokio::sync::Mutex as TokioMutex;
use crate::policy::{PolicyEngine, SecurityEvent, RuleAction, RuleType};
use crate::interceptors::{linux, windows};

/// Represents a detected security event that requires processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedEvent {
    pub id: u64,
    pub timestamp: DateTime<Utc>,
    pub event_type: String,
    pub process_name: String,
    pub target: String,
    pub action_taken: String,
    pub status: String, // allowed, blocked, pending
    pub details: String,
}

/// Main security engine that runs monitoring loops and processes events
pub struct SecurityEngine {
    policy_engine: Arc<TokioMutex<PolicyEngine>>,
    db: Arc<Mutex<Db>>,
    events: Arc<Mutex<Vec<DetectedEvent>>>,
    event_counter: Arc<Mutex<u64>>,
    is_running: Arc<Mutex<bool>>,
}

impl SecurityEngine {
    /// Create a new security engine instance
    pub fn new(policy_engine: Arc<TokioMutex<PolicyEngine>>, db: Arc<Mutex<Db>>) -> Self {
        SecurityEngine {
            policy_engine,
            db,
            events: Arc::new(Mutex::new(Vec::new())),
            event_counter: Arc::new(Mutex::new(0)),
            is_running: Arc::new(Mutex::new(false)),
        }
    }

    /// Start the security engine monitoring loops
    pub fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut is_running = self.is_running.lock().unwrap();
        if *is_running {
            return Err("Security engine is already running".into());
        }
        *is_running = true;

        // Clone arcs for use in threads
        let policy_engine_clone = Arc::clone(&self.policy_engine);
        let db_clone = Arc::clone(&self.db);
        let events_clone = Arc::clone(&self.events);
        let event_counter_clone = Arc::clone(&self.event_counter);
        let is_running_clone = Arc::clone(&self.is_running);

        // Start process monitoring thread
        let process_monitor_thread = thread::spawn(move || {
            Self::process_monitor_loop(
                policy_engine_clone,
                db_clone,
                events_clone,
                event_counter_clone,
                is_running_clone,
            )
        });

        // Start network monitoring thread
        let network_monitor_thread = thread::spawn(move || {
            Self::network_monitor_loop(
                policy_engine_clone,
                db_clone,
                events_clone,
                event_counter_clone,
                is_running_clone,
            )
        });

        // Start device monitoring thread
        let device_monitor_thread = thread::spawn(move || {
            Self::device_monitor_loop(
                policy_engine_clone,
                db_clone,
                events_clone,
                event_counter_clone,
                is_running_clone,
            )
        });

        // Wait for all threads to complete (they won't unless is_running becomes false)
        let _ = process_monitor_thread.join();
        let _ = network_monitor_thread.join();
        let _ = device_monitor_thread.join();

        Ok(())
    }

    /// Stop the security engine
    pub fn stop(&self) {
        let mut is_running = self.is_running.lock().unwrap();
        *is_running = false;
    }

    /// Get current security events for UI display
    pub fn get_events(&self) -> Vec<DetectedEvent> {
        let events = self.events.lock().unwrap();
        events.clone()
    }

    /// Process monitoring loop - watches for process creation/execution
    fn process_monitor_loop(
        policy_engine: Arc<TokioMutex<PolicyEngine>>,
        db: Arc<Mutex<Db>>,
        events: Arc<Mutex<Vec<DetectedEvent>>>,
        event_counter: Arc<Mutex<u64>>,
        is_running: Arc<Mutex<bool>>,
    ) {
        // In a real implementation, this would use platform-specific APIs
        // For Linux: ptrace, auditd, or eBPF to monitor execve syscalls
        // For Windows: PsSetCreateProcessNotifyRoutine or ETW

        // For now, we'll simulate events for demonstration
        let mut counter = 0;
        while *is_running.lock().unwrap() {
            // Sleep to avoid consuming too much CPU
            thread::sleep(Duration::from_millis(1000));

            // Simulate a process event every few seconds
            if counter % 5 == 0 {
                let mut event_counter = event_counter.lock().unwrap();
                *event_counter += 1;
                let event_id = *event_counter;
                drop(event_counter);

                let event = DetectedEvent {
                    id: event_id,
                    timestamp: Utc::now(),
                    event_type: "Process Execution".to_string(),
                    process_name: if counter % 10 == 0 { "suspicious_process.exe" } else { "notepad.exe" }.to_string(),
                    target: "C:\\Windows\\System32\\".to_string(),
                    action_taken: if counter % 10 == 0 { "Blocked" } else { "Allowed" }.to_string(),
                    status: if counter % 10 == 0 { "blocked" } else { "allowed" }.to_string(),
                    details: if counter % 10 == 0 {
                        "Attempted to execute from temp directory"
                    } else {
                        "Normal process execution"
                    }.to_string(),
                };

                // Add event to the list
                let mut events_list = events.lock().unwrap();
                events_list.push(event);
                // Keep only last 100 events to prevent memory growth
                if events_list.len() > 100 {
                    events_list.remove(0);
                }
                drop(events_list);
            }

            counter += 1;
        }
    }

    /// Network monitoring loop - watches for socket connections
    fn network_monitor_loop(
        policy_engine: Arc<TokioMutex<PolicyEngine>>,
        db: Arc<Mutex<Db>>,
        events: Arc<Mutex<Vec<DetectedEvent>>>,
        event_counter: Arc<Mutex<u64>>,
        is_running: Arc<Mutex<bool>>,
    ) {
        // In a real implementation, this would use:
        // For Linux: netlink sockets, eBPF, or nftables hooks
        // For Windows: Windows Filtering Platform (WFP) or TDI filters

        let mut counter = 0;
        while *is_running.lock().unwrap() {
            thread::sleep(Duration::from_millis(1500));

            // Simulate a network event every few seconds
            if counter % 7 == 0 {
                let mut event_counter = event_counter.lock().unwrap();
                *event_counter += 1;
                let event_id = *event_counter;
                drop(event_counter);

                let event = DetectedEvent {
                    id: event_id,
                    timestamp: Utc::now(),
                    event_type: "Network Connection".to_string(),
                    process_name: if counter % 15 == 0 { "curl.exe" } else { "chrome.exe" }.to_string(),
                    target: if counter % 15 == 0 { "93.184.216.34:80" } else { "8.8.8.8:53" }.to_string(),
                    action_taken: if counter % 15 == 0 { "Blocked" } else { "Allowed" }.to_string(),
                    status: if counter % 15 == 0 { "blocked" } else { "allowed" }.to_string(),
                    details: if counter % 15 == 0 {
                        "Connection to known malicious IP blocked"
                    } else {
                        "Normal DNS query"
                    }.to_string(),
                };

                // Add event to the list
                let mut events_list = events.lock().unwrap();
                events_list.push(event);
                // Keep only last 100 events
                if events_list.len() > 100 {
                    events_list.remove(0);
                }
                drop(events_list);
            }

            counter += 1;
        }
    }

    /// Device monitoring loop - watches for USB device mounting/unmounting
    fn device_monitor_loop(
        policy_engine: Arc<TokioMutex<PolicyEngine>>,
        db: Arc<Mutex<Db>>,
        events: Arc<Mutex<Vec<DetectedEvent>>>,
        event_counter: Arc<Mutex<u64>>,
        is_running: Arc<Mutex<bool>>,
    ) {
        // In a real implementation, this would use:
        // For Linux: udev rules or polling /sys/block
        // For Windows: RegisterDeviceNotification or WMI

        let mut counter = 0;
        while *is_running.lock().unwrap() {
            thread::sleep(Duration::from_millis(2000));

            // Simulate a device event every few seconds
            if counter % 10 == 0 {
                let mut event_counter = event_counter.lock().unwrap();
                *event_counter += 1;
                let event_id = *event_counter;
                drop(event_counter);

                let event = DetectedEvent {
                    id: event_id,
                    timestamp: Utc::now(),
                    event_type: "Device Mount".to_string(),
                    process_name: "system".to_string(),
                    target: if counter % 20 == 0 { "USB Mass Storage Device" } else { "USB Keyboard" }.to_string(),
                    action_taken: if counter % 20 == 0 { "Blocked" } else { "Allowed" }.to_string(),
                    status: if counter % 20 == 0 { "blocked" } else { "allowed" }.to_string(),
                    details: if counter % 20 == 0 {
                        "Untrusted USB storage device blocked"
                    } else {
                        "Standard USB input device allowed"
                    }.to_string(),
                };

                // Add event to the list
                let mut events_list = events.lock().unwrap();
                events_list.push(event);
                // Keep only last 100 events
                if events_list.len() > 100 {
                    events_list.remove(0);
                }
                drop(events_list);
            }

            counter += 1;
        }
    }

    /// Automatically handle dynamic whitelist approvals
    /// This would be called when a user approves an action through the UI
    pub fn handle_approval(&self, event_id: u64, action: ApprovalAction) -> Result<(), Box<dyn std::error::Error>> {
        // Find the event
        let mut events = self.events.lock().unwrap();
        if let Some(event) = events.iter_mut().find(|e| e.id == event_id) {
            match action {
                ApprovalAction::ApprovePermanently => {
                    // Add to permanent whitelist in Sled DB
                    let db = self.db.lock().unwrap();
                    let whitelist_key = format!("whitelist::{}", event.process_name);
                    db.insert(whitelist_key.as_bytes(), b"approved")?;
                    event.status = "allowed".to_string();
                    event.action_taken = "Approved Permanently".to_string();
                }
                ApprovalAction::TempAllow => {
                    // Add to temporary whitelist with expiry
                    let db = self.db.lock().unwrap();
                    let whitelist_key = format!("whitelist_temp::{}::{}", event.process_name, event.id);
                    // Store expiry time (15 minutes from now)
                    let expiry = (Utc::now() + chrono::Duration::minutes(15)).timestamp();
                    db.insert(whitelist_key.as_bytes(), expiry.to_string().as_bytes())?;
                    event.status = "allowed".to_string();
                    event.action_taken = "Temp Allow (15m)".to_string();
                }
                ApprovalAction::TerminateProcess => {
                    // In real implementation, this would terminate the process
                    event.status = "terminated".to_string();
                    event.action_taken = "Process Terminated".to_string();
                }
            }
        }
        Ok(())
    }

    /// Check if a process is in the whitelist (permanent or temporary)
    pub fn is_whitelisted(&self, process_name: &str) -> bool {
        let db = self.db.lock().unwrap();

        // Check permanent whitelist
        let perm_key = format!("whitelist::{}", process_name);
        if let Ok(Some(_)) = db.get(perm_key.as_bytes()) {
            return true;
        }

        // Check temporary whitelist
        let temp_prefix = format!("whitelist_temp::{}::", process_name);
        let scan = db.scan_prefix(temp_prefix.as_bytes());
        for item in scan {
            if let Ok((key, value)) = item {
                if let Ok(expiry_str) = std::str::from_utf8(&value) {
                    if let Ok(expiry) = expiry_str.parse::<i64>() {
                        if Utc::now().timestamp() < expiry {
                            return true; // Still valid
                        } else {
                            // Expired, remove it
                            let _ = db.remove(key);
                        }
                    }
                }
            }
        }

        false
    }
}

/// User approval actions for security events
#[derive(Debug, Clone, Copy)]
pub enum ApprovalAction {
    ApprovePermanently,
    TempAllow,
    TerminateProcess,
}

/// Background task to periodically clean up expired whitelist entries
pub fn start_whitelist_cleaner(db: Arc<Mutex<Db>>) {
    let db_clone = Arc::clone(&db);
    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_minutes(5)); // Check every 5 minutes

            let db = db_clone.lock().unwrap();
            let temp_prefix = b"whitelist_temp::";
            let scan = db.scan_prefix(temp_prefix);
            let mut expired_keys = Vec::new();

            for item in scan {
                if let Ok((key, value)) = item {
                    if let Ok(expiry_str) = std::str::from_utf8(&value) {
                        if let Ok(expiry) = expiry_str.parse::<i64>() {
                            if Utc::now().timestamp() >= expiry {
                                expired_keys.push(key.to_vec());
                            }
                        }
                    }
                }
            }

            // Remove expired entries
            for key in expired_keys {
                let _ = db.remove(key);
            }
        }
    });
}