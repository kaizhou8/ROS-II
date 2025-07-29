//! Sensor node module
//! 
//! Provides implementations of various sensor nodes including LiDAR, camera, IMU, etc.
//! Provides implementations for various sensor nodes including LiDAR, camera, IMU, etc.

use async_trait::async_trait;
use anyhow::Result;

use crate::core::{Node, NodeState, node::{BaseNode, NodeStats}};
use crate::messages::{Publisher, types::{SensorDataMessage, SensorData}};
use crate::time::Rate;

/// LiDAR sensor node
#[derive(Debug)]
pub struct LidarSensorNode {
    base: BaseNode,
    publisher: Option<Publisher>,
    rate: Rate,
    device_path: String,
}

impl LidarSensorNode {
    /// Create a new LiDAR sensor node
    pub fn new(name: String, namespace: String, device_path: String, frequency: f64) -> Self {
        Self {
            base: BaseNode::new(name, namespace),
            publisher: None,
            rate: Rate::new(frequency),
            device_path,
        }
    }
}

#[async_trait]
impl Node for LidarSensorNode {
    fn info(&self) -> &crate::core::NodeInfo {
        self.base.info()
    }
    
    fn state(&self) -> NodeState {
        self.base.state()
    }
    
    async fn initialize(&mut self) -> Result<()> {
        self.base.initialize().await?;
        if let Some(message_bus) = self.base.message_bus() {
            self.publisher = Some(message_bus.create_publisher("sensor/lidar").await?);
        }
        log::info!("LiDAR sensor node {} initialized", self.info().full_name());
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
            // TODO: Actual LiDAR data reading
            // TODO: Actual LiDAR data reading
            let data = SensorData::Scalar(100.0); // Placeholder data
            
            let message = SensorDataMessage::new(
                self.info().full_name(),
                "lidar".to_string(),
                "lidar_01".to_string(),
                data,
            );
            
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

/// Camera sensor node
#[derive(Debug)]
pub struct CameraSensorNode {
    base: BaseNode,
    publisher: Option<Publisher>,
    rate: Rate,
    camera_id: u32,
}

impl CameraSensorNode {
    /// Create a new camera sensor node
    pub fn new(name: String, namespace: String, camera_id: u32, frequency: f64) -> Self {
        Self {
            base: BaseNode::new(name, namespace),
            publisher: None,
            rate: Rate::new(frequency),
            camera_id,
        }
    }
}

#[async_trait]
impl Node for CameraSensorNode {
    fn info(&self) -> &crate::core::NodeInfo {
        self.base.info()
    }
    
    fn state(&self) -> NodeState {
        self.base.state()
    }
    
    async fn initialize(&mut self) -> Result<()> {
        self.base.initialize().await?;
        if let Some(message_bus) = self.base.message_bus() {
            self.publisher = Some(message_bus.create_publisher("sensor/camera").await?);
        }
        log::info!("Camera sensor node {} initialized", self.info().full_name());
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
            // TODO: Actual camera data reading
            // TODO: Actual camera data reading
            let data = SensorData::Vector3([640.0, 480.0, 3.0].into()); // Placeholder data (width, height, channels)
            
            let message = SensorDataMessage::new(
                self.info().full_name(),
                "camera".to_string(),
                format!("camera_{}", self.camera_id),
                data,
            );
            
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