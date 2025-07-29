//! Node handle for external node management
//! 
//! Provides a safe interface for managing nodes from outside the system.

use std::sync::Arc;
use tokio::sync::RwLock;
use anyhow::Result;

use super::{Node, NodeId, NodeInfo, NodeState};

/// Handle for managing a specific node
#[derive(Debug, Clone)]
pub struct NodeHandle {
    node_id: NodeId,
    node: Arc<RwLock<Box<dyn Node>>>,
}

impl NodeHandle {
    /// Create a new node handle
    pub fn new(node_id: NodeId, node: Arc<RwLock<Box<dyn Node>>>) -> Self {
        Self { node_id, node }
    }
    
    /// Get the node ID
    pub fn id(&self) -> NodeId {
        self.node_id
    }
    
    /// Get node information
    pub async fn info(&self) -> NodeInfo {
        let node = self.node.read().await;
        node.info().clone()
    }
    
    /// Get current node state
    pub async fn state(&self) -> NodeState {
        let node = self.node.read().await;
        node.state()
    }
    
    /// Start the node
    pub async fn start(&self) -> Result<()> {
        let mut node = self.node.write().await;
        node.start().await
    }
    
    /// Stop the node
    pub async fn stop(&self) -> Result<()> {
        let mut node = self.node.write().await;
        node.stop().await
    }
    
    /// Pause the node
    pub async fn pause(&self) -> Result<()> {
        let mut node = self.node.write().await;
        node.pause().await
    }
    
    /// Resume the node
    pub async fn resume(&self) -> Result<()> {
        let mut node = self.node.write().await;
        node.resume().await
    }
    
    /// Get node statistics
    pub async fn stats(&self) -> super::node::NodeStats {
        let node = self.node.read().await;
        node.get_stats()
    }
    
    /// Check if the node is running
    pub async fn is_running(&self) -> bool {
        self.state().await == NodeState::Running
    }
    
    /// Check if the node is stopped
    pub async fn is_stopped(&self) -> bool {
        matches!(self.state().await, NodeState::Stopped | NodeState::Error)
    }
    
    /// Wait for the node to reach a specific state
    pub async fn wait_for_state(&self, target_state: NodeState, timeout: std::time::Duration) -> Result<bool> {
        let start = std::time::Instant::now();
        
        while start.elapsed() < timeout {
            if self.state().await == target_state {
                return Ok(true);
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }
        
        Ok(false)
    }
}