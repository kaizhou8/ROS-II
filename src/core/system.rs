//! Robot system management
//! 
//! Manages the overall robot system, including node lifecycle and coordination.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc, broadcast};
use tokio::task::JoinHandle;
use anyhow::{Result, anyhow};

use super::{Node, NodeId, NodeInfo, NodeState, SystemConfig};
use crate::messages::MessageBus;
use crate::time::Time;

/// Main robot system that manages all nodes and services
#[derive(Debug)]
pub struct RobotSystem {
    config: SystemConfig,
    nodes: Arc<RwLock<HashMap<NodeId, Arc<RwLock<Box<dyn Node>>>>>>,
    node_handles: Arc<RwLock<HashMap<NodeId, JoinHandle<Result<()>>>>>,
    message_bus: MessageBus,
    shutdown_tx: broadcast::Sender<()>,
    shutdown_rx: broadcast::Receiver<()>,
    is_running: Arc<RwLock<bool>>,
    start_time: Time, // Add system start time
}

impl RobotSystem {
    /// Create a new robot system with default configuration
    pub async fn new() -> Result<Self> {
        Self::with_config(SystemConfig::default()).await
    }
    
    /// Create a new robot system with custom configuration
    pub async fn with_config(config: SystemConfig) -> Result<Self> {
        let (shutdown_tx, shutdown_rx) = broadcast::channel(16);
        let message_bus = MessageBus::new(config.message_buffer_size).await?;
        
        Ok(Self {
            config,
            nodes: Arc::new(RwLock::new(HashMap::new())),
            node_handles: Arc::new(RwLock::new(HashMap::new())),
            message_bus,
            shutdown_tx,
            shutdown_rx,
            is_running: Arc::new(RwLock::new(false)),
            start_time: Time::now(),
        })
    }
    
    /// Create a robot system from configuration file
    pub async fn from_config(config_path: &str) -> Result<Self> {
        let config = crate::config::load_config(config_path)?;
        Self::with_config(config).await
    }
    
    /// Add a node to the system
    pub async fn add_node(&self, mut node: Box<dyn Node>) -> Result<NodeId> {
        let node_id = node.info().id;
        let node_name = node.info().full_name();
        
        // Check if we've reached the maximum number of nodes
        {
            let nodes = self.nodes.read().await;
            if nodes.len() >= self.config.max_nodes {
                return Err(anyhow!("Maximum number of nodes ({}) reached", self.config.max_nodes));
            }
            
            // Check for name conflicts
            for existing_node in nodes.values() {
                let existing_node = existing_node.read().await;
                if existing_node.info().full_name() == node_name {
                    return Err(anyhow!("Node with name '{}' already exists", node_name));
                }
            }
        }
        
        // Set up the node with message bus
        if let Some(base_node) = node.as_any_mut().downcast_mut::<crate::core::node::BaseNode>() {
            base_node.set_message_bus(self.message_bus.clone());
        }
        
        // Initialize the node
        node.initialize().await?;
        
        // Store the node
        let node_arc = Arc::new(RwLock::new(node));
        self.nodes.write().await.insert(node_id, node_arc.clone());
        
        log::info!("Added node: {} (ID: {})", node_name, node_id);
        
        // Start the node if the system is running
        if *self.is_running.read().await {
            self.start_node_task(node_id, node_arc).await?;
        }
        
        Ok(node_id)
    }
    
    /// Remove a node from the system
    pub async fn remove_node(&self, node_id: NodeId) -> Result<()> {
        // Stop the node task if it's running
        if let Some(handle) = self.node_handles.write().await.remove(&node_id) {
            handle.abort();
        }
        
        // Remove and cleanup the node
        if let Some(node_arc) = self.nodes.write().await.remove(&node_id) {
            let mut node = node_arc.write().await;
            node.stop().await?;
            node.cleanup().await?;
            log::info!("Removed node: {} (ID: {})", node.info().full_name(), node_id);
        }
        
        Ok(())
    }
    
    /// Get node information
    pub async fn get_node_info(&self, node_id: NodeId) -> Option<NodeInfo> {
        let nodes = self.nodes.read().await;
        if let Some(node_arc) = nodes.get(&node_id) {
            let node = node_arc.read().await;
            Some(node.info().clone())
        } else {
            None
        }
    }
    
    /// List all nodes
    pub async fn list_nodes(&self) -> Vec<NodeInfo> {
        let nodes = self.nodes.read().await;
        let mut node_infos = Vec::new();
        
        for node_arc in nodes.values() {
            let node = node_arc.read().await;
            node_infos.push(node.info().clone());
        }
        
        node_infos
    }
    
    /// Start the robot system
    pub async fn start(&self) -> Result<()> {
        log::info!("Starting robot system...");
        
        *self.is_running.write().await = true;
        
        // Start all nodes
        let nodes = self.nodes.read().await;
        for (&node_id, node_arc) in nodes.iter() {
            self.start_node_task(node_id, node_arc.clone()).await?;
        }
        
        log::info!("Robot system started with {} nodes", nodes.len());
        Ok(())
    }
    
    /// Stop the robot system
    pub async fn stop(&self) -> Result<()> {
        log::info!("Stopping robot system...");
        
        *self.is_running.write().await = false;
        
        // Send shutdown signal
        let _ = self.shutdown_tx.send(());
        
        // Stop all node tasks
        let mut handles = self.node_handles.write().await;
        for (node_id, handle) in handles.drain() {
            handle.abort();
            log::debug!("Stopped task for node: {}", node_id);
        }
        
        // Stop all nodes
        let nodes = self.nodes.read().await;
        for node_arc in nodes.values() {
            let mut node = node_arc.write().await;
            if let Err(e) = node.stop().await {
                log::error!("Error stopping node {}: {}", node.info().full_name(), e);
            }
        }
        
        log::info!("Robot system stopped");
        Ok(())
    }
    
    /// Check if the system is running
    pub async fn is_running(&self) -> bool {
        *self.is_running.read().await
    }
    
    /// Get system statistics
    pub async fn get_system_stats(&self) -> SystemStats {
        let nodes = self.nodes.read().await;
        let mut stats = SystemStats {
            total_nodes: nodes.len(),
            running_nodes: 0,
            stopped_nodes: 0,
            error_nodes: 0,
            total_messages: 0,
            uptime: Time::now().duration_since(self.start_time)
                .and_then(|d| d.as_std_duration())
                .unwrap_or_default(),
        };
        
        for node_arc in nodes.values() {
            let node = node_arc.read().await;
            match node.state() {
                NodeState::Running => stats.running_nodes += 1,
                NodeState::Stopped => stats.stopped_nodes += 1,
                NodeState::Error => stats.error_nodes += 1,
                _ => {}
            }
            
            let node_stats = node.get_stats();
            stats.total_messages += node_stats.messages_received + node_stats.messages_sent;
        }
        
        stats
    }
    
    /// Wait for shutdown signal
    pub async fn wait_for_shutdown(&self) -> Result<()> {
        let mut rx = self.shutdown_tx.subscribe();
        rx.recv().await?;
        Ok(())
    }
    
    /// Run the system until shutdown
    pub async fn run(&self) -> Result<()> {
        self.start().await?;
        self.wait_for_shutdown().await?;
        self.stop().await?;
        Ok(())
    }
    
    /// Start a node task
    async fn start_node_task(
        &self,
        node_id: NodeId,
        node_arc: Arc<RwLock<Box<dyn Node>>>,
    ) -> Result<()> {
        let mut shutdown_rx = self.shutdown_tx.subscribe();
        
        let handle = tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(10));
            
            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        let mut node = node_arc.write().await;
                        if node.state() == NodeState::Running {
                            if let Err(e) = node.spin_once().await {
                                log::error!("Error in node {}: {}", node.info().full_name(), e);
                                // Could set node state to Error here
                            }
                        }
                    }
                    _ = shutdown_rx.recv() => {
                        log::debug!("Shutdown signal received for node: {}", node_id);
                        break;
                    }
                }
            }
            
            Ok(())
        });
        
        self.node_handles.write().await.insert(node_id, handle);
        Ok(())
    }
}

/// System-wide statistics
#[derive(Debug, Clone)]
pub struct SystemStats {
    pub total_nodes: usize,
    pub running_nodes: usize,
    pub stopped_nodes: usize,
    pub error_nodes: usize,
    pub total_messages: u64,
    pub uptime: std::time::Duration,
}

// Helper trait for downcasting
trait AsAny {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
}

impl<T: 'static> AsAny for T {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}