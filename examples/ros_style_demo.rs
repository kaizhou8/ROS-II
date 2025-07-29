use robot_framework_rust::{init_with_log_level, core::{Node, NodeState, node::{BaseNode, NodeStats}}};

use tokio::time::{sleep, Duration};
use async_trait::async_trait;
use anyhow::Result;

// Simple demo node that just logs messages
#[derive(Debug)]
struct DemoNode {
    base: BaseNode,
    counter: u32,
}

impl DemoNode {
    pub fn new(name: String, namespace: String) -> Self {
        Self {
            base: BaseNode::new(name, namespace),
            counter: 0,
        }
    }
}

#[async_trait]
impl Node for DemoNode {
    async fn initialize(&mut self) -> Result<()> {
        self.base.set_state(NodeState::Running);
        log::info!("Demo node {} initialized", self.base.info().full_name());
        Ok(())
    }
    
    async fn start(&mut self) -> Result<()> {
        self.base.set_state(NodeState::Running);
        log::info!("Demo node {} started", self.base.info().full_name());
        Ok(())
    }
    
    async fn stop(&mut self) -> Result<()> {
        self.base.set_state(NodeState::Stopped);
        log::info!("Demo node {} stopped", self.base.info().full_name());
        Ok(())
    }
    
    async fn pause(&mut self) -> Result<()> {
        self.base.set_state(NodeState::Paused);
        log::info!("Demo node {} paused", self.base.info().full_name());
        Ok(())
    }
    
    async fn resume(&mut self) -> Result<()> {
        self.base.set_state(NodeState::Running);
        log::info!("Demo node {} resumed", self.base.info().full_name());
        Ok(())
    }
    
    async fn spin_once(&mut self) -> Result<()> {
        if self.state() != NodeState::Running {
            return Ok(());
        }
        
        self.counter += 1;
        log::info!("Demo node {} tick #{}", self.base.info().full_name(), self.counter);
        
        // Simulate some work
        sleep(Duration::from_millis(100)).await;
        
        self.base.update_stats(|stats| {
            stats.messages_sent += 1;
        });
        
        Ok(())
    }
    
    fn state(&self) -> NodeState {
        self.base.state()
    }
    
    fn info(&self) -> &robot_framework_rust::core::NodeInfo {
        self.base.info()
    }
    
    fn get_stats(&self) -> NodeStats {
        self.base.get_stats()
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize the robot system
    let _system = init_with_log_level(log::LevelFilter::Info).await?;
    
    log::info!("🤖 ROS-Style Robot Framework Demo");
    log::info!("=== Simple Node Demo ===");
    
    // Create and run a simple demo node
    let mut demo_node = DemoNode::new("demo_node".to_string(), "/demo".to_string());
    
    // Initialize the node
    demo_node.initialize().await?;
    
    // Start the node
    demo_node.start().await?;
    
    // Run the node for a few iterations
    log::info!("🚀 Running demo node for 5 seconds...");
    for i in 1..=50 {
        demo_node.spin_once().await?;
        if i % 10 == 0 {
            let stats = demo_node.get_stats();
            log::info!("📊 Node stats: messages_sent={}, cpu_usage={:.2}%", 
                stats.messages_sent, stats.cpu_usage_percent);
        }
    }
    
    // Stop the node
    demo_node.stop().await?;
    
    log::info!("✅ Demo completed successfully!");
    log::info!("=== Final Stats ===");
    let final_stats = demo_node.get_stats();
    log::info!("Total messages sent: {}", final_stats.messages_sent);
    log::info!("Average execution time: {:.2} μs", final_stats.average_execution_time_us);
    
    Ok(())
}