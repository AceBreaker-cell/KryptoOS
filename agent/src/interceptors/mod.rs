#[cfg(target_os = "linux")]
pub mod linux;

#[cfg(target_os = "windows")]
pub mod windows;

/// Platform-independent interface for starting the interceptor worker
#[cfg(target_os = "linux")]
pub use linux::start_interceptor_worker;

#[cfg(target_os = "windows")]
pub use windows::start_interceptor_worker;