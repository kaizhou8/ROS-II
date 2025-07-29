//! Sensor fusion example
//! 
//! Demonstrates how to combine data from multiple sensors.

use anyhow::Result;
use async_trait::async_trait;
use std::collections::VecDeque;
use nalgebra::Vector3;

use robot_framework_rust::{
    init,
    core::{Node, NodeState, node::{BaseNode, NodeStats}},
    messages::{
        Publisher, Subscription,
        types::{SensorDataMessage, SensorData}
    },
    time::Rate,
    nodes::{SensorNode, RandomDataGenerator},
};

/// Sensor fusion node that combines multiple sensor inputs
#[derive(Debug)]
pub struct SensorFusionNode {
    base: BaseNode,
    imu_subscription: Option<Subscription>,
    gps_subscription: Option<Subscription>,
    fusion_publisher: Option<Publisher>,
    rate: Rate,
    imu_buffer: VecDeque<Vector3<f64>>,
    gps_buffer: VecDeque<Vector3<f64>>,
    buffer_size: usize,
}

impl SensorFusionNode {
    pub fn new(name: String, namespace: String, frequency: f64) -> Self {
        Self {
            base: BaseNode::new(name, namespace),
            imu_subscription: None,
            gps_subscription: None,
            fusion_publisher: None,
            rate: Rate::new(frequency),
            imu_buffer: VecDeque::new(),
            gps_buffer: VecDeque::new(),
            buffer_size: 10,
        }
    }
    
    async fn setup_communications(&mut self) -> Result<()> {
        if let Some(message_bus) = self.base.message_bus() {
            // Subscribe to sensor topics
            self.imu_subscription = Some(message_bus.subscribe("sensor/imu").await?);
            self.gps_subscription = Some(message_bus.subscribe("sensor/gps").await?);
            
            // Create publisher for fused data
            self.fusion_publisher = Some(message_bus.create_publisher("sensor/fused").await?);
        }
        Ok(())
    }
    
    fn process_sensor_data(&mut self, data: &SensorData, sensor_type: &str) {
        if let SensorData::Vector3(vec) = data {
            match sensor_type {
                "imu" => {
                    self.imu_buffer.push_back(*vec);
                    if self.imu_buffer.len() > self.buffer_size {
                        self.imu_buffer.pop_front();
                    }
                }
                "gps" => {
                    self.gps_buffer.push_back(*vec);
                    if self.gps_buffer.len() > self.buffer_size {
                        self.gps_buffer.pop_front();
                    }
                }
                _ => {}
            }
        }
    }
    
    fn compute_fused_position(&self) -> Option<Vector3<f64>> {
        if self.imu_buffer.is_empty() || self.gps_buffer.is_empty() {
            return None;
        }
        
        // Simple fusion: weighted average of latest readings
        let imu_weight = 0.3;
        let gps_weight = 0.7;
        
        let latest_imu = self.imu_buffer.back()?;
        let latest_gps = self.gps_buffer.back()?;
        
        Some(latest_imu * imu_weight + latest_gps * gps_weight)
    }
}

#[async_trait]
impl Node for SensorFusionNode {
    fn info(&self) -> &robot_framework_rust::core::NodeInfo {
        self.base.info()
    }
    
    fn state(&self) -> NodeState {
        self.base.state()
    }
    
    async fn initialize(&mut self) -> Result<()> {
        self.base.initialize().await?;
        self.setup_communications().await?;
        log::info!("Sensor fusion node {} initialized", self.info().full_name());
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
        if self.state() != NodeState::Running {
            return Ok(());
        }
        
        // Collect messages to process (avoiding borrow conflicts)
        let mut imu_messages = Vec::new();
        let mut gps_messages = Vec::new();
        
        // Process incoming IMU messages
        if let Some(imu_sub) = &mut self.imu_subscription {
            while let Ok(message) = imu_sub.try_recv() {
                if let Some(sensor_msg) = message.as_any().downcast_ref::<SensorDataMessage>() {
                    imu_messages.push(sensor_msg.data.clone());
                }
            }
        }
        
        // Process incoming GPS messages
        if let Some(gps_sub) = &mut self.gps_subscription {
            while let Ok(message) = gps_sub.try_recv() {
                if let Some(sensor_msg) = message.as_any().downcast_ref::<SensorDataMessage>() {
                    gps_messages.push(sensor_msg.data.clone());
                }
            }
        }
        
        // Now process all collected messages
        for data in imu_messages {
            self.process_sensor_data(&data, "imu");
        }
        
        for data in gps_messages {
            self.process_sensor_data(&data, "gps");
        }
        
        // Perform sensor fusion and publish result
        if !self.imu_buffer.is_empty() && !self.gps_buffer.is_empty() {
            if let Some(fused_position) = self.compute_fused_position() {
                if let Some(publisher) = &self.fusion_publisher {
                    let fused_msg = SensorDataMessage::new(
                        self.info().full_name(),
                        "fused".to_string(),
                        "fusion_01".to_string(),
                        SensorData::Vector3(fused_position),
                    );
                    
                    if let Err(e) = publisher.publish(fused_msg).await {
                        log::warn!("Failed to publish fused data: {}", e);
                    } else {
                        self.base.update_stats(|stats| {
                            stats.messages_sent += 1;
                        });
                    }
                }
            }
        }
        
        self.rate.sleep().await;
        Ok(())
    }
    
    fn get_stats(&self) -> NodeStats {
        self.base.get_stats()
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("🔄 Sensor Fusion Example");
    
    // Initialize the robot system (this will also initialize logging)
    let system = init().await?;
    
    // Create IMU sensor
    let imu_sensor = SensorNode::new(
        "imu_sensor".to_string(),
        "/sensors".to_string(),
        "imu".to_string(),
        "imu_01".to_string(),
        50.0, // 50 Hz
        Box::new(RandomDataGenerator::new("position".to_string())),
    );
    
    // Create GPS sensor
    let gps_sensor = SensorNode::new(
        "gps_sensor".to_string(),
        "/sensors".to_string(),
        "gps".to_string(),
        "gps_01".to_string(),
        10.0, // 10 Hz
        Box::new(RandomDataGenerator::new("position".to_string())),
    );
    
    // Create sensor fusion node
    let fusion_node = SensorFusionNode::new(
        "sensor_fusion".to_string(),
        "/fusion".to_string(),
        20.0, // 20 Hz
    );
    
    // Add nodes to the system
    let imu_id = system.add_node(Box::new(imu_sensor)).await?;
    let gps_id = system.add_node(Box::new(gps_sensor)).await?;
    let fusion_id = system.add_node(Box::new(fusion_node)).await?;
    
    println!("✅ Created nodes:");
    println!("   - IMU Sensor ({})", imu_id);
    println!("   - GPS Sensor ({})", gps_id);
    println!("   - Sensor Fusion ({})", fusion_id);
    
    // Start the system
    system.start().await?;
    println!("🚀 Sensor fusion system started!");
    
    // Let it run for 15 seconds
    println!("📊 Running sensor fusion for 15 seconds...");
    tokio::time::sleep(tokio::time::Duration::from_secs(15)).await;
    
    // Print statistics
    let stats = system.get_system_stats().await;
    println!("📈 Final statistics:");
    println!("   - Total messages: {}", stats.total_messages);
    println!("   - Running nodes: {}", stats.running_nodes);
    
    // Stop the system
    system.stop().await?;
    println!("🛑 Sensor fusion system stopped");
    
    Ok(())
}