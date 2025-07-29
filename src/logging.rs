//! Logging utilities for the robot framework
//! 
//! Provides structured logging with different levels and output formats.

use log::{Level, LevelFilter};
use std::io::Write;
use chrono::{DateTime, Utc};

/// Initialize the default logger
pub fn init_default_logger() {
    // Use try_init to avoid panic if logger is already initialized
    let _ = env_logger::Builder::from_default_env()
        .format(|buf, record| {
            let timestamp = Utc::now().format("%Y-%m-%d %H:%M:%S%.3f");
            writeln!(
                buf,
                "[{}] [{}] [{}:{}] {}",
                timestamp,
                record.level(),
                record.file().unwrap_or("unknown"),
                record.line().unwrap_or(0),
                record.args()
            )
        })
        .filter_level(LevelFilter::Info)
        .try_init();
}

/// Initialize logger with custom level
pub fn init_logger_with_level(level: LevelFilter) {
    // Use try_init to avoid panic if logger is already initialized
    let _ = env_logger::Builder::from_default_env()
        .format(|buf, record| {
            let timestamp = Utc::now().format("%Y-%m-%d %H:%M:%S%.3f");
            writeln!(
                buf,
                "[{}] [{}] [{}:{}] {}",
                timestamp,
                record.level(),
                record.file().unwrap_or("unknown"),
                record.line().unwrap_or(0),
                record.args()
            )
        })
        .filter_level(level)
        .try_init();
}

/// Initialize logger for robotics with node information
pub fn init_robot_logger(node_name: &str, level: LevelFilter) {
    let node_name_owned = node_name.to_owned();
    // Use try_init to avoid panic if logger is already initialized
    let _ = env_logger::Builder::from_default_env()
        .format(move |buf, record| {
            let timestamp = Utc::now().format("%Y-%m-%d %H:%M:%S%.3f");
            writeln!(
                buf,
                "[{}] [{}] [{}] [{}:{}] {}",
                timestamp,
                record.level(),
                &node_name_owned,
                record.file().unwrap_or("unknown"),
                record.line().unwrap_or(0),
                record.args()
            )
        })
        .filter_level(level)
        .try_init();
}

/// Macro for logging with node context
#[macro_export]
macro_rules! node_log {
    ($level:expr, $node:expr, $($arg:tt)*) => {
        log::log!($level, "[{}] {}", $node, format_args!($($arg)*));
    };
}

/// Convenience macros for different log levels with node context
#[macro_export]
macro_rules! node_error {
    ($node:expr, $($arg:tt)*) => {
        $crate::node_log!(log::Level::Error, $node, $($arg)*);
    };
}

#[macro_export]
macro_rules! node_warn {
    ($node:expr, $($arg:tt)*) => {
        $crate::node_log!(log::Level::Warn, $node, $($arg)*);
    };
}

#[macro_export]
macro_rules! node_info {
    ($node:expr, $($arg:tt)*) => {
        $crate::node_log!(log::Level::Info, $node, $($arg)*);
    };
}

#[macro_export]
macro_rules! node_debug {
    ($node:expr, $($arg:tt)*) => {
        $crate::node_log!(log::Level::Debug, $node, $($arg)*);
    };
}

#[macro_export]
macro_rules! node_trace {
    ($node:expr, $($arg:tt)*) => {
        $crate::node_log!(log::Level::Trace, $node, $($arg)*);
    };
}