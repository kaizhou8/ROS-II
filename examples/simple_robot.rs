//! Simple robot example
//! 
//! Demonstrates basic usage of the robot framework.

use anyhow::Result;
use robot_framework_rust::{
    init,
    nodes::{SensorNode, SystemMonitorNode, RandomDataGenerator},
};

#[tokio::main]
async fn main() -> Result<()> {
    println!("🤖 Simple Robot Example");
    
    // Initialize the robot system (this will also initialize logging)
    let system = init().await?;
    
    // Create a temperature sensor
    let temp_sensor = SensorNode::new(
        "temp_sensor".to_string(),
        "/".to_string(),
        "temperature".to_string(),
        "temp_01".to_string(),
        5.0, // 5 Hz
        Box::new(RandomDataGenerator::new("temperature".to_string())),
    );
    
    // Create a system monitor
    let monitor = SystemMonitorNode::new(
        "monitor".to_string(),
        "/".to_string(),
        1.0, // 1 Hz
    );
    
    // Add nodes to the system
    let temp_id = system.add_node(Box::new(temp_sensor)).await?;
    let monitor_id = system.add_node(Box::new(monitor)).await?;
    
    println!("✅ Created nodes: temp_sensor ({}), monitor ({})", temp_id, monitor_id);
    
    // Start the system
    system.start().await?;
    println!("🚀 System started!");
    
    // Let it run for 10 seconds
    tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
    
    // Print statistics
    let stats = system.get_system_stats().await;
    println!("📊 Messages processed: {}", stats.total_messages);
    
    // Stop the system
    system.stop().await?;
    println!("🛑 System stopped");
    
    Ok(())
}