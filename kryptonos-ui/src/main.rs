use kryptonos_core::{
    engine::{ApprovalAction, SecurityEngine, start_whitelist_cleaner},
    policy::PolicyEngine,
    ipc::start_ipc_server,
    assets::logo::create_logo_bitmap,
};
use slint::{ComponentHandle, ModelRc, SharedString, StandardListView, VecModel};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tokio::sync::Mutex as TokioMutex;
use tracing::{info, error, warn, Level};
use tracing_subscriber::FmtSubscriber;

// Import the generated UI components
slint::include_modules!();

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize asynchronous logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set up tracing subscriber");

    info!("Starting KryptonOS Unified Application");

    // Initialize Sled DB engine
    let db_path = "/var/lib/kryptonos/db";
    if let Err(e) = std::fs::create_dir_all(db_path) {
        error!("Failed to create DB directory: {}", e);
        std::process::exit(1);
    }

    let db = Arc::new(Mutex::new(
        sled::open(format!("{}/rules.db", db_path))
            .map_err(|e| format!("Failed to open Sled DB: {}", e))?,
    ));

    // Initialize policy engine
    let policy_engine = Arc::new(TokioMutex::new(PolicyEngine::new(Arc::clone(&db))));

    // Initialize default rules in the database
    {
        let policy_engine_lock = policy_engine.blocking_lock();
        let _ = policy_engine_lock.initialize_default_rules();
    }

    // Create security engine
    let security_engine = Arc::new(SecurityEngine::new(
        Arc::clone(&policy_engine),
        Arc::clone(&db),
    ));

    // Start the whitelist cleaner thread
    start_whitelist_cleaner(Arc::clone(&db));

    // Start security engine monitoring loops in a separate thread
    let security_engine_clone = Arc::clone(&security_engine);
    let engine_thread = thread::spawn(move || {
        if let Err(e) = security_engine_clone.start() {
            error!("Failed to start security engine: {}", e);
        }
    });

    // Set up IPC server for communication (if needed for other integrations)
    let ipc_policy_engine = Arc::clone(&policy_engine);
    let ipc_thread = thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        if let Err(e) = rt.block_on(start_ipc_server(ipc_policy_engine)) {
            error!("IPC server failed: {}", e);
        }
    });

    // Load and display the UI
    let ui = AppWindow::new()?;

    // Set up UI components and event handlers
    setup_ui(&ui, Arc::clone(&security_engine));

    // Show the window
    ui.show()?;

    // Run the SLint event loop
    slint::run_event_loop()?;

    // Cleanup
    let _ = engine_thread.join();
    let _ = ipc_thread.join();

    info!("KryptonOS shutting down");
    Ok(())
}

/// Set up the UI components and connect them to the security engine
fn setup_ui(ui: &AppWindowHandle, security_engine: Arc<SecurityEngine>) {
    // Set up the security mode switch
    ui.on_toggle_default_deny(move |enabled| {
        // In a real implementation, this would send a command to the engine
        // For now, we'll just update the UI state
        ui.set_system_status(if enabled {
            SharedString::from("Protected")
        } else {
            SharedString::from("Vulnerable - Default Deny Disabled")
        });
    });

    // Set up initial UI state
    ui.set_default_deny_enabled(true);
    ui.set_system_status(SharedString::from("Protected"));

    // Set up event handling for security alerts
    // We'll periodically update the UI with new events
    let ui_weak = ui.as_weak();
    let security_engine_weak = Arc::downgrade(&security_engine);

    std::thread::spawn(move || {
        loop {
            std::thread::sleep(std::time::Duration::from_millis(1000));

            // Update event count and list
            if let (Some(ui), Some(engine)) = (ui_weak.upgrade(), security_engine_weak.upgrade()) {
                let events = engine.get_events();
                let alert_count = events.iter().filter(|e| e.status == "blocked").count();

                ui.set_alert_count(alert_count as i32);

                // Update the events list in the UI
                // In a real implementation with Slint, we would use a model
                // For simplicity, we're just updating a counter here
                // A full implementation would use slint::ModelRc<VecModel<EventItemData>>
            }
        }
    });
}

// Define the data structure for events that will be displayed in the UI
#[derive(Debug, Clone)]
struct EventItemData {
    process_name: SharedString,
    action: SharedString,
    target: SharedString,
    time: SharedString,
    status: SharedString,
}

// In a more complete implementation, we would connect this to a Slint ListView
// For this example, we're keeping it simple with periodic updates