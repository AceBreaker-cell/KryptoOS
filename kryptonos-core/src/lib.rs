pub mod assets;
pub mod engine;
pub mod interceptors;
pub mod ipc;
pub mod policy;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_creation() {
        use sled::Config;
        use std::sync::Arc;
        use tokio::sync::Mutex;

        let config = Config::new().temporary(true);
        let db = Arc::new(Mutex::new(config.open().unwrap()));
        let policy_engine = Arc::new(Mutex::new(super::policy::PolicyEngine::new(db.clone())));
        let engine = super::engine::SecurityEngine::new(policy_engine, db);
        assert!(!engine.events.lock().unwrap().is_empty() == false); // Initially empty
    }
}