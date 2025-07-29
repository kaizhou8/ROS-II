//! Node trait and base implementations
//! 
//! Defines the core Node trait that all robot nodes must implement.

use async_trait::async_trait;
use std::fmt::Debug;
use tokio::sync::mpsc;
use anyhow::Result;

use super::{NodeId, NodeInfo};
use crate::messages::{Message, MessageBus};
use crate::time::Time;

/// Node lifecycle states
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeState {
    Created,
    Initializing,
    Running,
    Paused,
    Stopping,
    Stopped,
    Error,
}

impl std::fmt::Display for NodeState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NodeState::Created => write!(f, "Created"),
            NodeState::Initializing => write!(f, "Initializing"),
            NodeState::Running => write!(f, "Running"),
            NodeState::Paused => write!(f, "Paused"),
            NodeState::Stopping => write!(f, "Stopping"),
            NodeState::Stopped => write!(f, "Stopped"),
            NodeState::Error => write!(f, "Error"),
        }
    }
}

/// Core trait that all nodes must implement
#[async_trait]
pub trait Node: Send + Sync + Debug {
    /// Get node information
    fn info(&self) -> &NodeInfo;
    
    /// Get current node state
    fn state(&self) -> NodeState;
    
    /// Initialize the node
    async fn initialize(&mut self) -> Result<()>;
    
    /// Start the node execution
    async fn start(&mut self) -> Result<()>;
    
    /// Stop the node execution
    async fn stop(&mut self) -> Result<()>;
    
    /// Pause the node execution
    async fn pause(&mut self) -> Result<()>;
    
    /// Resume the node execution
    async fn resume(&mut self) -> Result<()>;
    
    /// Main execution loop - called repeatedly while running
    async fn spin_once(&mut self) -> Result<()>;
    
    /// Cleanup resources when node is destroyed
    async fn cleanup(&mut self) -> Result<()> {
        Ok(())
    }
    
    /// Handle incoming messages (optional override)
    async fn handle_message(&mut self, _message: Box<dyn Message>) -> Result<()> {
        Ok(())
    }
    
    /// Get node statistics (optional override)
    fn get_stats(&self) -> NodeStats {
        NodeStats::default()
    }
}

/// Node execution statistics
#[derive(Debug, Clone, Default)]
pub struct NodeStats {
    pub messages_received: u64,
    pub messages_sent: u64,
    pub errors: u64,
    pub last_execution_time_us: u64,
    pub average_execution_time_us: f64,
    pub cpu_usage_percent: f32,
    pub memory_usage_bytes: u64,
}

/// Base node implementation providing common functionality
#[derive(Debug)]
pub struct BaseNode {
    info: NodeInfo,
    state: NodeState,
    message_bus: Option<MessageBus>,
    stats: NodeStats,
    shutdown_tx: Option<mpsc::Sender<()>>,
    shutdown_rx: Option<mpsc::Receiver<()>>,
}

impl BaseNode {
    /// Create a new base node
    pub fn new(name: String, namespace: String) -> Self {
        let (shutdown_tx, shutdown_rx) = mpsc::channel(1);
        
        Self {
            info: NodeInfo::new(name, namespace),
            state: NodeState::Created,
            message_bus: None,
            stats: NodeStats::default(),
            shutdown_tx: Some(shutdown_tx),
            shutdown_rx: Some(shutdown_rx),
        }
    }
    
    /// Set the message bus for this node
    pub fn set_message_bus(&mut self, message_bus: MessageBus) {
        self.message_bus = Some(message_bus);
    }
    
    /// Get a reference to the message bus
    pub fn message_bus(&self) -> Option<&MessageBus> {
        self.message_bus.as_ref()
    }
    
    /// Update node state
    pub fn set_state(&mut self, state: NodeState) {
        log::debug!("Node {} state changed: {} -> {}", 
                   self.info.full_name(), self.state, state);
        self.state = state;
    }
    
    /// Get shutdown receiver
    pub fn take_shutdown_receiver(&mut self) -> Option<mpsc::Receiver<()>> {
        self.shutdown_rx.take()
    }
    
    /// Request shutdown
    pub async fn request_shutdown(&self) -> Result<()> {
        if let Some(tx) = &self.shutdown_tx {
            tx.send(()).await?;
        }
        Ok(())
    }
    
    /// Update statistics
    pub fn update_stats<F>(&mut self, f: F) 
    where 
        F: FnOnce(&mut NodeStats)
    {
        f(&mut self.stats);
    }
}

#[async_trait]
impl Node for BaseNode {
    fn info(&self) -> &NodeInfo {
        &self.info
    }
    
    fn state(&self) -> NodeState {
        self.state
    }
    
    async fn initialize(&mut self) -> Result<()> {
        self.set_state(NodeState::Initializing);
        log::info!("Initializing node: {}", self.info.full_name());
        self.set_state(NodeState::Running);
        Ok(())
    }
    
    async fn start(&mut self) -> Result<()> {
        if self.state != NodeState::Running {
            self.set_state(NodeState::Running);
        }
        log::info!("Starting node: {}", self.info.full_name());
        Ok(())
    }
    
    async fn stop(&mut self) -> Result<()> {
        self.set_state(NodeState::Stopping);
        log::info!("Stopping node: {}", self.info.full_name());
        self.request_shutdown().await?;
        self.set_state(NodeState::Stopped);
        Ok(())
    }
    
    async fn pause(&mut self) -> Result<()> {
        self.set_state(NodeState::Paused);
        log::info!("Pausing node: {}", self.info.full_name());
        Ok(())
    }
    
    async fn resume(&mut self) -> Result<()> {
        self.set_state(NodeState::Running);
        log::info!("Resuming node: {}", self.info.full_name());
        Ok(())
    }
    
    async fn spin_once(&mut self) -> Result<()> {
        // Default implementation does nothing
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        Ok(())
    }
    
    fn get_stats(&self) -> NodeStats {
        self.stats.clone()
    }
}