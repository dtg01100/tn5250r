//! Component health signals registry
//!
//! Provides a simple global, thread-safe registry for component-specific
//! health signals that other monitors (like IntegrationMonitor) can query.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use super::integration_monitor::ComponentState;

#[derive(Debug, Clone, Default)]
pub struct ComponentSignal {
    pub state: ComponentState,
    pub is_critical: bool,
    pub last_error: Option<String>,
    pub metrics: HashMap<String, String>,
}

#[derive(Debug, Default)]
struct RegistryInner {
    map: HashMap<String, ComponentSignal>,
}

static REGISTRY: once_cell::sync::Lazy<Arc<Mutex<RegistryInner>>> =
    once_cell::sync::Lazy::new(|| Arc::new(Mutex::new(RegistryInner::default())));

/// Set/update a component's state
pub fn set_component_status(name: &str, state: ComponentState) {
    if let Ok(mut inner) = REGISTRY.lock() {
        let entry = inner.map.entry(name.to_string()).or_default();
        entry.state = state;
    }
}

/// Mark a component critical or not
pub fn set_component_critical(name: &str, is_critical: bool) {
    if let Ok(mut inner) = REGISTRY.lock() {
        let entry = inner.map.entry(name.to_string()).or_default();
        entry.is_critical = is_critical;
    }
}

/// Attach a metric value to a component
pub fn set_component_metric(name: &str, key: &str, value: impl ToString) {
    if let Ok(mut inner) = REGISTRY.lock() {
        let entry = inner.map.entry(name.to_string()).or_default();
        entry.metrics.insert(key.to_string(), value.to_string());
    }
}

/// Attach an error string to a component
pub fn set_component_error(name: &str, err: Option<impl ToString>) {
    if let Ok(mut inner) = REGISTRY.lock() {
        let entry = inner.map.entry(name.to_string()).or_default();
        entry.last_error = err.map(|e| e.to_string());
    }
}

/// Get a snapshot of a component signal
pub fn get_component_signal(name: &str) -> Option<ComponentSignal> {
    REGISTRY
        .lock()
        .ok()
        .and_then(|inner| inner.map.get(name).cloned())
}

/// Get snapshot of all component signals
pub fn get_all_signals() -> HashMap<String, ComponentSignal> {
    REGISTRY
        .lock()
        .ok()
        .map(|inner| inner.map.clone())
        .unwrap_or_default()
}

/// Clear registry (primarily for tests)
#[cfg(test)]
pub fn clear_registry() {
    if let Ok(mut inner) = REGISTRY.lock() {
        inner.map.clear();
    }
}

/// Whether the app is running in a headless/CI mode
/// Based on env var TN5250R_HEADLESS=1
pub fn is_headless() -> bool {
    std::env::var("TN5250R_HEADLESS").map(|v| v == "1").unwrap_or(false)
}
