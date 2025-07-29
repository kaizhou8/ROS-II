//! Error handling for Robot Framework Rust

use std::fmt;

/// Result type alias for Robot Framework operations
pub type Result<T> = std::result::Result<T, RobotError>;

/// Robot Framework error types
#[derive(Debug)]
pub enum RobotError {
    /// Configuration errors
    Config(String),
    /// Node-related errors
    Node(String),
    /// Message handling errors
    Message(String),
    /// Service errors
    Service(String),
    /// Memory management errors
    Memory(String),
    /// Time-related errors
    Time(String),
    /// Transform errors
    Transform(String),
    /// Action execution errors
    Action(String),
    /// Logging errors
    Logging(String),
    /// Debug system errors
    Debug(String),
    /// Visualization errors
    Visualization(String),
    /// Profiling errors
    Profiling(String),
    /// I/O errors
    Io(String),
    /// Serialization errors
    Serialization(String),
    /// Other errors
    Other(String),
}

impl Clone for RobotError {
    fn clone(&self) -> Self {
        match self {
            RobotError::Config(msg) => RobotError::Config(msg.clone()),
            RobotError::Node(msg) => RobotError::Node(msg.clone()),
            RobotError::Message(msg) => RobotError::Message(msg.clone()),
            RobotError::Service(msg) => RobotError::Service(msg.clone()),
            RobotError::Memory(msg) => RobotError::Memory(msg.clone()),
            RobotError::Time(msg) => RobotError::Time(msg.clone()),
            RobotError::Transform(msg) => RobotError::Transform(msg.clone()),
            RobotError::Action(msg) => RobotError::Action(msg.clone()),
            RobotError::Logging(msg) => RobotError::Logging(msg.clone()),
            RobotError::Debug(msg) => RobotError::Debug(msg.clone()),
            RobotError::Visualization(msg) => RobotError::Visualization(msg.clone()),
            RobotError::Profiling(msg) => RobotError::Profiling(msg.clone()),
            RobotError::Io(msg) => RobotError::Io(msg.clone()),
            RobotError::Serialization(msg) => RobotError::Serialization(msg.clone()),
            RobotError::Other(msg) => RobotError::Other(msg.clone()),
        }
    }
}

impl fmt::Display for RobotError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RobotError::Config(msg) => write!(f, "Configuration error: {}", msg),
            RobotError::Node(msg) => write!(f, "Node error: {}", msg),
            RobotError::Message(msg) => write!(f, "Message error: {}", msg),
            RobotError::Service(msg) => write!(f, "Service error: {}", msg),
            RobotError::Memory(msg) => write!(f, "Memory error: {}", msg),
            RobotError::Time(msg) => write!(f, "Time error: {}", msg),
            RobotError::Transform(msg) => write!(f, "Transform error: {}", msg),
            RobotError::Action(msg) => write!(f, "Action error: {}", msg),
            RobotError::Logging(msg) => write!(f, "Logging error: {}", msg),
            RobotError::Debug(msg) => write!(f, "Debug error: {}", msg),
            RobotError::Visualization(msg) => write!(f, "Visualization error: {}", msg),
            RobotError::Profiling(msg) => write!(f, "Profiling error: {}", msg),
            RobotError::Io(msg) => write!(f, "I/O error: {}", msg),
            RobotError::Serialization(msg) => write!(f, "Serialization error: {}", msg),
            RobotError::Other(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl std::error::Error for RobotError {}

impl From<std::io::Error> for RobotError {
    fn from(err: std::io::Error) -> Self {
        RobotError::Io(err.to_string())
    }
}

impl From<serde_json::Error> for RobotError {
    fn from(err: serde_json::Error) -> Self {
        RobotError::Serialization(err.to_string())
    }
}

impl From<anyhow::Error> for RobotError {
    fn from(err: anyhow::Error) -> Self {
        RobotError::Other(err.to_string())
    }
}