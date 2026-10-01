use std::path::PathBuf;
use std::sync::Arc;
use tokio::net::UnixListener;
use tokio::sync::{Mutex, broadcast};
use tracing::{info, error, warn, Level};
use tracing_subscriber::FmtSubscriber;
use chrono::Utc;

mod policy;
mod ipc;
mod interceptors;

#[tokio::main]
async fn main() {
    // Initialize asynchronous logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set up tracing subscriber");

    info!("Starting KryptonOS System Agent");

    // Initialize Sled DB engine
    let db_path = PathBuf::from("/var/lib/kryptonos/db");
    if let Err(e) = std::fs::create_dir_all(&db_path) {
        error!("Failed to create DB directory: {}", e);
        std::process::exit(1);
    }

    let db = match sled::open(db_path.join("rules.db")) {
        Ok(db) => Arc::new(Mutex::new(db)),
        Err(e) => {
            error!("Failed to open Sled DB: {}", e);
            std::process::exit(1);
        }
    };

    // Spawn Kernel Interception Worker
    let interception_worker = tokio::spawn(interceptors::start_interceptor_worker(Arc::clone(&db)));

    // Spin up Local IPC Server thread
    let ipc_server = tokio::spawn(ipc::start_ipc_server(Arc::clone(&db)));

    // Wait for either task to complete (or fail)
    tokio::select! {
        res = interception_worker => {
            if let Err(e) = res {
                error!("Interception worker failed: {}", e);
            }
        }
        res = ipc_server => {
            if let Err(e) = res {
                error!("IPC server failed: {}", e);
            }
        }
    }

    info!("KryptonOS System Agent shutting down");
}