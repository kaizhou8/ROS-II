//! Robot Framework Rust
//! 
//! A high-performance, memory-safe robotics framework built with Rust.
//! This framework provides the core building blocks for creating robust
//! robotics applications with real-time capabilities.

pub mod actions;
pub mod config;
pub mod core;
pub mod error;
pub mod logging;
pub mod memory;
pub mod messages;
pub mod nodes;
pub mod services;
pub mod time;
pub mod transforms;

// Re-export commonly used types
pub use crate::core::{NodeId, NodeInfo, SystemConfig};
pub use crate::core::node::NodeStats;
pub use crate::core::system::{RobotSystem, SystemStats};
pub use crate::error::{RobotError, Result};
pub use crate::memory::{MemoryPool, RingBuffer, MemoryUsage, MemoryMonitor, TrackedBox, StringInterner};

// Re-export for convenience
pub use async_trait::async_trait;
pub use tokio;

/// Framework version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Framework name
pub const NAME: &str = env!("CARGO_PKG_NAME");

/// Initialize the robot framework with default settings
pub async fn init() -> anyhow::Result<RobotSystem> {
    logging::init_default_logger();
    log::info!("Robot Framework Rust v{} initialized", VERSION);
    RobotSystem::new().await
}

/// Initialize the robot framework with custom log level
pub async fn init_with_log_level(level: log::LevelFilter) -> anyhow::Result<RobotSystem> {
    logging::init_logger_with_level(level);
    log::info!("Robot Framework Rust v{} initialized with log level {:?}", VERSION, level);
    RobotSystem::new().await
}

/// Initialize the robot framework from a configuration file
pub async fn init_with_config(config_path: &str) -> anyhow::Result<RobotSystem> {
    logging::init_default_logger();
    log::info!("Robot Framework Rust v{} initializing from config: {}", VERSION, config_path);
    RobotSystem::from_config(config_path).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
        assert!(!NAME.is_empty());
    }

    #[test]
    fn test_init() {
        // Test that init doesn't panic
        let _ = init();
    }

    #[test]
    fn test_init_with_log_level() {
        // Test that init_with_log_level doesn't panic
        let _ = init_with_log_level(log::LevelFilter::Info);
    }
}