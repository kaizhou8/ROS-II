//! Example robot nodes for common robotics tasks
//! 
//! Provides ready-to-use node implementations for typical robot operations.

use async_trait::async_trait;
use anyhow::Result;
use crate::core::{Node, NodeState, node::{BaseNode, NodeStats}};
use crate::messages::{
    Publisher,
    types::{SensorDataMessage, SensorData, SystemStatusMessage, SystemStatus}
};
use crate::time::Rate;

pub mod sensor;
pub mod control;
pub mod navigation;
pub mod perception;

pub use sensor::*;
pub use control::*;
pub use navigation::*;
pub use perception::*;

/// Simple sensor node that publishes sensor data
#[derive(Debug)]
pub struct SensorNode {
    base: BaseNode,
    publisher: Option<Publisher>,
    sensor_type: String,
    sensor_id: String,
    rate: Rate,
    data_generator: Box<dyn SensorDataGenerator + Send + Sync>,
}

impl SensorNode {
    /// Create a new sensor node
    pub fn new(
        name: String,
        namespace: String,
        sensor_type: String,
        sensor_id: String,
        frequency: f64,
        data_generator: Box<dyn SensorDataGenerator + Send + Sync>,
    ) -> Self {
        Self {
            base: BaseNode::new(name, namespace),
            publisher: None,
            sensor_type,
            sensor_id,
            rate: Rate::new(frequency),
            data_generator,
        }
    }
    
    /// Set up the publisher
    async fn setup_publisher(&mut self) -> Result<()> {
        if let Some(message_bus) = self.base.message_bus() {
            let topic = format!("sensor/{}", self.sensor_type);
            self.publisher = Some(message_bus.create_publisher(topic).await?);
        }
        Ok(())
    }
}

#[async_trait]
impl Node for SensorNode {
    fn info(&self) -> &crate::core::NodeInfo {
        self.base.info()
    }
    
    fn state(&self) -> NodeState {
        self.base.state()
    }
    
    async fn initialize(&mut self) -> Result<()> {
        self.base.initialize().await?;
        self.setup_publisher().await?;
        log::info!("Sensor node {} initialized", self.info().full_name());
        Ok(())
    }
    
    async fn start(&mut self) -> Result<()> {
        self.base.start().await
    }
    
    async fn stop(&mut self) -> Result<()> {
        self.base.stop().await
    }
    
    async fn pause(&mut self) -> Result<()> {
        self.base.pause().await
    }
    
    async fn resume(&mut self) -> Result<()> {
        self.base.resume().await
    }
    
    async fn spin_once(&mut self) -> Result<()> {
        if self.state() == NodeState::Running {
            // Generate sensor data
            let data = self.data_generator.generate().await?;
            
            // Create and publish message
            let message = SensorDataMessage::new(
                self.info().full_name(),
                self.sensor_type.clone(),
                self.sensor_id.clone(),
                data,
            );
            
            if let Some(publisher) = &self.publisher {
                publisher.publish(message).await?;
                
                // Update statistics
                self.base.update_stats(|stats| {
                    stats.messages_sent += 1;
                });
            }
            
            // Rate limiting
            self.rate.sleep().await;
        }
        
        Ok(())
    }
    
    fn get_stats(&self) -> NodeStats {
        self.base.get_stats()
    }
}

/// Trait for generating sensor data
#[async_trait]
pub trait SensorDataGenerator: std::fmt::Debug {
    async fn generate(&mut self) -> Result<SensorData>;
}

/// Simple random data generator for testing
#[derive(Debug)]
pub struct RandomDataGenerator {
    data_type: String,
}

impl RandomDataGenerator {
    pub fn new(data_type: String) -> Self {
        Self { data_type }
    }
}

#[async_trait]
impl SensorDataGenerator for RandomDataGenerator {
    async fn generate(&mut self) -> Result<SensorData> {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        
        match self.data_type.as_str() {
            "temperature" => Ok(SensorData::Scalar(rng.gen_range(20.0..30.0))),
            "position" => {
                let pos = nalgebra::Vector3::new(
                    rng.gen_range(-10.0..10.0),
                    rng.gen_range(-10.0..10.0),
                    rng.gen_range(0.0..5.0),
                );
                Ok(SensorData::Vector3(pos))
            }
            _ => Ok(SensorData::Scalar(rng.gen_range(0.0..100.0))),
        }
    }
}

/// System monitor node that publishes system status
#[derive(Debug)]
pub struct SystemMonitorNode {
    base: BaseNode,
    publisher: Option<Publisher>,
    rate: Rate,
}

impl SystemMonitorNode {
    /// Create a new system monitor node
    pub fn new(name: String, namespace: String, frequency: f64) -> Self {
        Self {
            base: BaseNode::new(name, namespace),
            publisher: None,
            rate: Rate::new(frequency),
        }
    }
    
    /// Set up the publisher
    async fn setup_publisher(&mut self) -> Result<()> {
        if let Some(message_bus) = self.base.message_bus() {
            self.publisher = Some(message_bus.create_publisher("system/status").await?);
        }
        Ok(())
    }
    
    /// Get system metrics
    fn get_system_metrics(&self) -> (f32, f64, f32) {
        // Simplified system metrics - in a real implementation,
        // you would use system APIs to get actual metrics
        use rand::Rng;
        let mut rng = rand::thread_rng();
        
        let cpu_usage = rng.gen_range(10.0..80.0);
        let memory_usage = rng.gen_range(100_000_000.0..1_000_000_000.0);
        let disk_usage = rng.gen_range(20.0..90.0);
        
        (cpu_usage, memory_usage, disk_usage)
    }
}

#[async_trait]
impl Node for SystemMonitorNode {
    fn info(&self) -> &crate::core::NodeInfo {
        self.base.info()
    }
    
    fn state(&self) -> NodeState {
        self.base.state()
    }
    
    async fn initialize(&mut self) -> Result<()> {
        self.base.initialize().await?;
        self.setup_publisher().await?;
        log::info!("System monitor node {} initialized", self.info().full_name());
        Ok(())
    }
    
    async fn start(&mut self) -> Result<()> {
        self.base.start().await
    }
    
    async fn stop(&mut self) -> Result<()> {
        self.base.stop().await
    }
    
    async fn pause(&mut self) -> Result<()> {
        self.base.pause().await
    }
    
    async fn resume(&mut self) -> Result<()> {
        self.base.resume().await
    }
    
    async fn spin_once(&mut self) -> Result<()> {
        if self.state() == NodeState::Running {
            let (cpu, memory, disk) = self.get_system_metrics();
            
            let status = if cpu > 90.0 || memory > 900_000_000.0 || disk > 95.0 {
                SystemStatus::Critical
            } else if cpu > 70.0 || memory > 700_000_000.0 || disk > 80.0 {
                SystemStatus::Warning
            } else {
                SystemStatus::Healthy
            };
            
            let message = SystemStatusMessage::new(
                self.info().full_name(),
                self.info().name.clone(),
                status,
            ).with_resources(cpu, memory, disk);
            
            if let Some(publisher) = &self.publisher {
                publisher.publish(message).await?;
                
                self.base.update_stats(|stats| {
                    stats.messages_sent += 1;
                });
            }
            
            self.rate.sleep().await;
        }
        
        Ok(())
    }
    
    fn get_stats(&self) -> NodeStats {
        self.base.get_stats()
    }
}