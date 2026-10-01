pub mod policy;
pub mod ipc;
pub mod interceptors;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_policy_engine_creation() {
        use sled::Config;
        use std::sync::Arc;
        use tokio::sync::Mutex;

        let config = Config::new().temporary(true);
        let db = Arc::new(Mutex::new(config.open().unwrap()));
        let engine = super::policy::PolicyEngine::new(db);
        assert!(&engine.db.is_locked());
    }
}