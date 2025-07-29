//! Main application entry point
//! 
//! Demonstrates the robot framework with example nodes.

use anyhow::Result;
use log::LevelFilter;
use robot_framework_rust::{
    init, 
    nodes::{SensorNode, SystemMonitorNode, RandomDataGenerator},
    logging::init_logger_with_level,
};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    init_logger_with_level(LevelFilter::Info);
    
    println!("🤖 Robot Framework Rust - Starting...");
    
    // Initialize the robot system
    let system = init().await?;
    
    // Create example nodes
    let sensor_node = SensorNode::new(
        "temperature_sensor".to_string(),
        "/sensors".to_string(),
        "temperature".to_string(),
        "temp_01".to_string(),
        10.0, // 10 Hz
        Box::new(RandomDataGenerator::new("temperature".to_string())),
    );
    
    let position_sensor = SensorNode::new(
        "position_sensor".to_string(),
        "/sensors".to_string(),
        "position".to_string(),
        "pos_01".to_string(),
        20.0, // 20 Hz
        Box::new(RandomDataGenerator::new("position".to_string())),
    );
    
    let monitor_node = SystemMonitorNode::new(
        "system_monitor".to_string(),
        "/system".to_string(),
        1.0, // 1 Hz
    );
    
    // Add nodes to the system
    let temp_id = system.add_node(Box::new(sensor_node)).await?;
    let pos_id = system.add_node(Box::new(position_sensor)).await?;
    let monitor_id = system.add_node(Box::new(monitor_node)).await?;
    
    println!("✅ Added nodes:");
    println!("   - Temperature Sensor (ID: {})", temp_id);
    println!("   - Position Sensor (ID: {})", pos_id);
    println!("   - System Monitor (ID: {})", monitor_id);
    
    // Start the system
    system.start().await?;
    println!("🚀 Robot system started!");
    
    // Run for a while to demonstrate functionality
    println!("📊 Running for 30 seconds...");
    tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
    
    // Print system statistics
    let stats = system.get_system_stats().await;
    println!("\n📈 System Statistics:");
    println!("   - Total nodes: {}", stats.total_nodes);
    println!("   - Running nodes: {}", stats.running_nodes);
    println!("   - Total messages: {}", stats.total_messages);
    
    // List all nodes
    let nodes = system.list_nodes().await;
    println!("\n📋 Active Nodes:");
    for node_info in nodes {
        println!("   - {} ({}): {}", 
                node_info.full_name(), 
                node_info.id, 
                node_info.state);
    }
    
    // Graceful shutdown
    println!("\n🛑 Shutting down...");
    system.stop().await?;
    println!("✅ Robot system stopped successfully!");
    
    Ok(())
}

/// Example of a more complex robot application
#[allow(dead_code)]
async fn complex_robot_example() -> Result<()> {
    use robot_framework_rust::config::{RobotConfig, NodeConfig, ParameterValue};

    
    // Create a configuration
    let mut config = RobotConfig::default();
    
    // Configure system parameters
    config.system.max_nodes = 50;
    config.system.message_buffer_size = 2000;
    config.system.heartbeat_interval_ms = 500;
    
    // Add node configurations
    let mut sensor_config = NodeConfig::default();
    sensor_config.enabled = true;
    sensor_config.namespace = "/sensors".to_string();
    sensor_config.rate_hz = Some(50.0);
    sensor_config.parameters.insert(
        "sensor_type".to_string(), 
        ParameterValue::String("lidar".to_string())
    );
    
    config.nodes.insert("lidar_sensor".to_string(), sensor_config);
    
    // Save configuration
    robot_framework_rust::config::save_config(&config, "robot_config.toml")?;
    
    // Load and use configuration
    let _system = robot_framework_rust::init_with_config("robot_config.toml").await?;
    
    // ... rest of the application
    
    Ok(())
}