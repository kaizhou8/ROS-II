//! Configuration management for the robot framework
//! 
//! Handles loading and parsing of configuration files.

use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use anyhow::{Result, Context};

use crate::core::SystemConfig;

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RobotConfig {
    pub system: SystemConfig,
    pub nodes: HashMap<String, NodeConfig>,
    pub topics: HashMap<String, TopicConfig>,
    pub parameters: HashMap<String, ParameterValue>,
}

/// Node-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeConfig {
    pub enabled: bool,
    pub namespace: String,
    pub parameters: HashMap<String, ParameterValue>,
    pub subscriptions: Vec<String>,
    pub publications: Vec<String>,
    pub rate_hz: Option<f64>,
}

/// Topic configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicConfig {
    pub message_type: String,
    pub buffer_size: Option<usize>,
    pub qos: QosConfig,
}

/// Quality of Service configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QosConfig {
    pub reliability: ReliabilityPolicy,
    pub durability: DurabilityPolicy,
    pub history: HistoryPolicy,
    pub depth: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReliabilityPolicy {
    BestEffort,
    Reliable,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DurabilityPolicy {
    Volatile,
    TransientLocal,
    Transient,
    Persistent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HistoryPolicy {
    KeepLast,
    KeepAll,
}

/// Parameter value types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ParameterValue {
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Array(Vec<ParameterValue>),
    Object(HashMap<String, ParameterValue>),
}

impl Default for RobotConfig {
    fn default() -> Self {
        Self {
            system: SystemConfig::default(),
            nodes: HashMap::new(),
            topics: HashMap::new(),
            parameters: HashMap::new(),
        }
    }
}

impl Default for NodeConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            namespace: "/".to_string(),
            parameters: HashMap::new(),
            subscriptions: Vec::new(),
            publications: Vec::new(),
            rate_hz: None,
        }
    }
}

impl Default for QosConfig {
    fn default() -> Self {
        Self {
            reliability: ReliabilityPolicy::Reliable,
            durability: DurabilityPolicy::Volatile,
            history: HistoryPolicy::KeepLast,
            depth: 10,
        }
    }
}

impl Default for TopicConfig {
    fn default() -> Self {
        Self {
            message_type: "Unknown".to_string(),
            buffer_size: None,
            qos: QosConfig::default(),
        }
    }
}

/// Load configuration from file
pub fn load_config(path: &str) -> Result<SystemConfig> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read config file: {}", path))?;
    
    let robot_config: RobotConfig = if path.ends_with(".toml") {
        toml::from_str(&content)
            .with_context(|| format!("Failed to parse TOML config: {}", path))?
    } else if path.ends_with(".json") {
        serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse JSON config: {}", path))?
    } else {
        return Err(anyhow::anyhow!("Unsupported config file format. Use .toml or .json"));
    };
    
    Ok(robot_config.system)
}

/// Load full robot configuration
pub fn load_robot_config(path: &str) -> Result<RobotConfig> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read config file: {}", path))?;
    
    let robot_config: RobotConfig = if path.ends_with(".toml") {
        toml::from_str(&content)
            .with_context(|| format!("Failed to parse TOML config: {}", path))?
    } else if path.ends_with(".json") {
        serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse JSON config: {}", path))?
    } else {
        return Err(anyhow::anyhow!("Unsupported config file format. Use .toml or .json"));
    };
    
    Ok(robot_config)
}

/// Save configuration to file
pub fn save_config(config: &RobotConfig, path: &str) -> Result<()> {
    let content = if path.ends_with(".toml") {
        toml::to_string_pretty(config)
            .with_context(|| "Failed to serialize config to TOML")?
    } else if path.ends_with(".json") {
        serde_json::to_string_pretty(config)
            .with_context(|| "Failed to serialize config to JSON")?
    } else {
        return Err(anyhow::anyhow!("Unsupported config file format. Use .toml or .json"));
    };
    
    std::fs::write(path, content)
        .with_context(|| format!("Failed to write config file: {}", path))?;
    
    Ok(())
}

/// Parameter server for runtime parameter management
#[derive(Debug)]
pub struct ParameterServer {
    parameters: HashMap<String, ParameterValue>,
}

impl ParameterServer {
    /// Create a new parameter server
    pub fn new() -> Self {
        Self {
            parameters: HashMap::new(),
        }
    }
    
    /// Load parameters from configuration
    pub fn from_config(config: &RobotConfig) -> Self {
        Self {
            parameters: config.parameters.clone(),
        }
    }
    
    /// Set a parameter
    pub fn set_parameter(&mut self, name: String, value: ParameterValue) {
        self.parameters.insert(name, value);
    }
    
    /// Get a parameter
    pub fn get_parameter(&self, name: &str) -> Option<&ParameterValue> {
        self.parameters.get(name)
    }
    
    /// Get a parameter as a specific type
    pub fn get_bool(&self, name: &str) -> Option<bool> {
        match self.get_parameter(name)? {
            ParameterValue::Bool(value) => Some(*value),
            _ => None,
        }
    }
    
    pub fn get_int(&self, name: &str) -> Option<i64> {
        match self.get_parameter(name)? {
            ParameterValue::Int(value) => Some(*value),
            _ => None,
        }
    }
    
    pub fn get_float(&self, name: &str) -> Option<f64> {
        match self.get_parameter(name)? {
            ParameterValue::Float(value) => Some(*value),
            _ => None,
        }
    }
    
    pub fn get_string(&self, name: &str) -> Option<&String> {
        match self.get_parameter(name)? {
            ParameterValue::String(value) => Some(value),
            _ => None,
        }
    }
    
    /// Remove a parameter
    pub fn remove_parameter(&mut self, name: &str) -> Option<ParameterValue> {
        self.parameters.remove(name)
    }
    
    /// List all parameter names
    pub fn list_parameters(&self) -> Vec<&String> {
        self.parameters.keys().collect()
    }
    
    /// Check if a parameter exists
    pub fn has_parameter(&self, name: &str) -> bool {
        self.parameters.contains_key(name)
    }
}

impl Default for ParameterServer {
    fn default() -> Self {
        Self::new()
    }
}