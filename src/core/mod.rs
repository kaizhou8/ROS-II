//! Core framework components
//! 
//! This module contains the fundamental building blocks of the robot framework.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use uuid::Uuid;
use anyhow::Result;
use serde::{Serialize, Deserialize};

pub mod node;
pub mod system;
pub mod handle;

pub use node::{Node, NodeState};
pub use system::RobotSystem;
pub use handle::NodeHandle;

/// Unique identifier for nodes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(pub Uuid);

impl NodeId {
    /// Create a new random node ID
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for NodeId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Node information structure
#[derive(Debug, Clone)]
pub struct NodeInfo {
    pub id: NodeId,
    pub name: String,
    pub namespace: String,
    pub state: NodeState,
    pub start_time: crate::time::Time,
}

impl NodeInfo {
    pub fn new(name: String, namespace: String) -> Self {
        Self {
            id: NodeId::new(),
            name,
            namespace,
            state: NodeState::Created,
            start_time: crate::time::Time::now(),
        }
    }

    /// Get the fully qualified node name
    pub fn full_name(&self) -> String {
        if self.namespace.is_empty() || self.namespace == "/" {
            format!("/{}", self.name)
        } else {
            format!("{}/{}", self.namespace, self.name)
        }
    }
}

/// System-wide configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemConfig {
    pub max_nodes: usize,
    pub message_buffer_size: usize,
    pub heartbeat_interval_ms: u64,
    pub node_timeout_ms: u64,
    pub log_level: String,
}

impl Default for SystemConfig {
    fn default() -> Self {
        Self {
            max_nodes: 1000,
            message_buffer_size: 1000,
            heartbeat_interval_ms: 1000,
            node_timeout_ms: 5000,
            log_level: "info".to_string(),
        }
    }
}