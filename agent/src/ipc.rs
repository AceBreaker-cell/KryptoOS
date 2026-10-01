use std::path::Path;
use std::sync::Arc;
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::{Mutex, broadcast};
use tracing::{info, error, warn, debug};
use crate::{policy::{PolicyEngine, SecurityEvent, RuleAction}, security_policy::SecurityRule};
use serde::{Deserialize, Serialize};
use serde_json::json;

/// IPC message types for communication between agent and dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IpcMessage {
    /// Request to evaluate a security event
    EvaluateEvent(SecurityEvent),
    /// Response with the action to take
    EvaluateResponse(RuleAction),
    /// Request to get current system status
    GetStatus,
    /// Response with system status
    StatusResponse(SystemStatus),
    /// Request to add/update a security rule
    UpdateRule(security_policy::SecurityRule),
    /// Response confirming rule update
    RuleUpdateResponse(bool),
    /// Request to get audit logs
    GetAuditLog(u64), // offset
    /// Response with audit log entries
    AuditLogResponse(Vec<AuditEntry>),
    /// Heartbeat/ping message
    Ping,
    /// Pong response
    Pong,
}

/// System status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemStatus {
    pub is_active: bool,
    pub rules_count: u64,
    pub events_processed: u64,
    pub last_event_time: Option<String>,
    pub version: String,
}

/// Audit log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub timestamp: String,
    pub event_type: String,
    pub action_taken: String,
    pub details: String,
}

/// IPC server handling communication over UNIX domain socket
pub struct IpcServer {
    socket_path: String,
    policy_engine: Arc<Mutex<PolicyEngine>>,
    event_broadcast: broadcast::Sender<IpcMessage>,
}

impl IpcServer {
    /// Create a new IPC server instance
    pub fn new(socket_path: String, policy_engine: Arc<Mutex<PolicyEngine>>) -> Self {
        IpcServer {
            socket_path,
            policy_engine,
            event_broadcast: broadcast::channel(100).0,
        }
    }

    /// Start the IPC server
    pub async fn start(self) -> Result<(), Box<dyn std::error::Error>> {
        // Remove old socket file if it exists
        let socket_path = Path::new(&self.socket_path);
        if socket_path.exists() {
            let _ = std::fs::remove_file(socket_path);
        }

        // Create directory for socket if needed
        if let Some(parent) = socket_path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)?;
            }
        }

        // Bind to UNIX domain socket
        let listener = UnixListener::bind(socket_path)?;
        info!("IPC server listening on {}", self.socket_path);

        // Accept incoming connections
        loop {
            match listener.accept().await {
                Ok((stream, addr)) => {
                    debug!("Accepted connection from {:?}", addr);
                    let policy_engine = Arc::clone(&self.policy_engine);
                    let mut broadcast_rx = self.event_broadcast.subscribe();

                    tokio::spawn(async move {
                        if let Err(e) = Self::handle_connection(stream, policy_engine, &mut broadcast_rx).await {
                            error!("Error handling connection: {}", e);
                        }
                    });
                }
                Err(e) => {
                    error!("Failed to accept connection: {}", e);
                }
            }
        }
    }

    /// Handle an individual IPC connection
    async fn handle_connection(
        mut stream: UnixStream,
        policy_engine: Arc<Mutex<PolicyEngine>>,
        broadcast_rx: &mut broadcast::Receiver<IpcMessage>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};

        // Buffer for reading messages
        let mut buffer = vec![0; 4096];

        loop {
            tokio::select! {
                // Read incoming message
                result = stream.read(&mut buffer) => {
                    match result {
                        Ok(0) => {
                            // Connection closed
                            debug!("Client disconnected");
                            break;
                        }
                        Ok(n) => {
                            // Parse message
                            let message_data = &buffer[..n];
                            if let Ok(message) = serde_json::from_slice::<IpcMessage>(message_data) {
                                let response = Self::process_message(message, &policy_engine).await?;

                                // Send response
                                let response_data = serde_json::to_vec(&response)?;
                                stream.write_all(&response_data).await?;
                                stream.write_all(b"\n").await?; // Newline delimiter
                            } else {
                                warn!("Failed to parse IPC message");
                                // Send error response
                                let error_msg = IpcMessage::EvaluateResponse(RuleAction::Deny);
                                let error_data = serde_json::to_vec(&error_msg)?;
                                stream.write_all(&error_data).await?;
                            }
                        }
                        Err(e) => {
                            error!("Error reading from socket: {}", e);
                            break;
                        }
                    }
                }
                // Broadcast outgoing events
                result = broadcast_rx.recv() => {
                    match result {
                        Ok(message) => {
                            let message_data = serde_json::to_vec(&message)?;
                            stream.write_all(&message_data).await?;
                            stream.write_all(b"\n").await?;
                        }
                        Err(e) => {
                            error!("Error receiving broadcast: {}", e);
                            break;
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Process an incoming IPC message
    async fn process_message(
        message: IpcMessage,
        policy_engine: &Arc<Mutex<PolicyEngine>>,
    ) -> Result<IpcMessage, Box<dyn std::error::Error>> {
        match message {
            IpcMessage::EvaluateEvent(event) => {
                let action = {
                    let engine = policy_engine.lock().await;
                    engine.evaluate_event(&event).await
                };
                Ok(IpcMessage::EvaluateResponse(action))
            }
            IpcMessage::GetStatus => {
                let status = Self::get_system_status(policy_engine).await?;
                Ok(IpcMessage::StatusResponse(status))
            }
            IpcMessage::UpdateRule(rule) => {
                let success = Self::update_rule(policy_engine, rule).await?;
                Ok(IpcMessage::RuleUpdateResponse(success))
            }
            IpcMessage::GetAuditLog(offset) => {
                let entries = Self::get_audit_log(policy_engine, offset).await?;
                Ok(IpcMessage::AuditLogResponse(entries))
            }
            IpcMessage::Ping => Ok(IpcMessage::Pong),
            _ => Ok(IpcMessage::EvaluateResponse(RuleAction::Deny)), // Default response
        }
    }

    /// Get system status
    async fn get_system_status(policy_engine: &Arc<Mutex<PolicyEngine>>) -> Result<SystemStatus, Box<dyn std::error::Error>> {
        let engine = policy_engine.lock().await;
        let db = engine.db.lock().await;

        // Get rules count
        let rules_count = match db.get(b"rules::count") {
            Ok(Some(val)) => String::from_utf8_lossy(&val).parse::<u64>().unwrap_or(0),
            Ok(None) => 0,
            Err(_) => 0,
        };

        Ok(SystemStatus {
            is_active: true,
            rules_count,
            events_processed: 0, // Would be tracked in a real implementation
            last_event_time: None,
            version: env!("CARGO_PKG_VERSION").to_string(),
        })
    }

    /// Update a security rule
    async fn update_rule(
        policy_engine: &Arc<Mutex<PolicyEngine>>,
        rule: security_policy::SecurityRule,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        // In a real implementation, this would store the rule in the DB
        // For now, we'll just return success
        Ok(true)
    }

    /// Get audit log entries
    async fn get_audit_log(
        policy_engine: &Arc<Mutex<PolicyEngine>>,
        offset: u64,
    ) -> Result<Vec<AuditEntry>, Box<dyn std::error::Error>> {
        // In a real implementation, this would fetch from DB
        // For now, return empty vec
        Ok(vec![])
    }
}

/// Start the IPC server as a task
pub async fn start_ipc_server(policy_engine: Arc<Mutex<PolicyEngine>>) -> Result<(), Box<dyn std::error::Error>> {
    let server = IpcServer::new("/var/run/kryptonos.sock".to_string(), policy_engine);
    server.start().await
}