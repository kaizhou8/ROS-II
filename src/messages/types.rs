//! Common message types for robotics applications
//! 
//! Provides standard message types commonly used in robotics.

use std::any::Any;
use serde::{Serialize, Deserialize};
use nalgebra::{Vector3, Quaternion, Point3};

use super::{Message, MessageId, BaseMessage};

/// 3D position and orientation (pose)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pose3D {
    pub position: Point3<f64>,
    pub orientation: Quaternion<f64>,
}

impl Default for Pose3D {
    fn default() -> Self {
        Self {
            position: Point3::origin(),
            orientation: Quaternion::identity(),
        }
    }
}

/// 3D velocity (linear and angular)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Twist3D {
    pub linear: Vector3<f64>,
    pub angular: Vector3<f64>,
}

impl Default for Twist3D {
    fn default() -> Self {
        Self {
            linear: Vector3::zeros(),
            angular: Vector3::zeros(),
        }
    }
}

/// Robot action command message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RobotActionMessage {
    pub base: BaseMessage,
    pub action_type: String,
    pub parameters: std::collections::HashMap<String, String>,
    pub target_pose: Option<Pose3D>,
    pub target_velocity: Option<Twist3D>,
    pub duration_ms: Option<u64>,
    pub priority: u8,
}

impl RobotActionMessage {
    pub fn new(source: String, action_type: String) -> Self {
        Self {
            base: BaseMessage::new(source, "robot/action".to_string()),
            action_type,
            parameters: std::collections::HashMap::new(),
            target_pose: None,
            target_velocity: None,
            duration_ms: None,
            priority: 5,
        }
    }
    
    pub fn with_pose(mut self, pose: Pose3D) -> Self {
        self.target_pose = Some(pose);
        self
    }
    
    pub fn with_velocity(mut self, velocity: Twist3D) -> Self {
        self.target_velocity = Some(velocity);
        self
    }
    
    pub fn with_duration(mut self, duration_ms: u64) -> Self {
        self.duration_ms = Some(duration_ms);
        self
    }
    
    pub fn with_priority(mut self, priority: u8) -> Self {
        self.priority = priority;
        self
    }
}

impl Message for RobotActionMessage {
    fn type_name(&self) -> &'static str {
        "RobotActionMessage"
    }
    
    fn id(&self) -> MessageId {
        self.base.id
    }
    
    fn timestamp(&self) -> crate::time::Time {
        self.base.timestamp
    }
    
    fn source(&self) -> &str {
        &self.base.source
    }
    
    fn topic(&self) -> &str {
        &self.base.topic
    }
    
    fn serialize(&self) -> anyhow::Result<Vec<u8>> {
        Ok(bincode::serialize(self)?)
    }
    
    fn clone_box(&self) -> Box<dyn Message> {
        Box::new(self.clone())
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Sensor data message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorDataMessage {
    pub base: BaseMessage,
    pub sensor_type: String,
    pub sensor_id: String,
    pub data: SensorData,
    pub quality: f32,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SensorData {
    Scalar(f64),
    Vector3(Vector3<f64>),
    Pose(Pose3D),
    Image { width: u32, height: u32, data: Vec<u8> },
    PointCloud(Vec<Point3<f64>>),
    LaserScan { ranges: Vec<f32>, angle_min: f32, angle_max: f32, angle_increment: f32 },
    Custom(Vec<u8>),
}

impl SensorDataMessage {
    pub fn new(source: String, sensor_type: String, sensor_id: String, data: SensorData) -> Self {
        Self {
            base: BaseMessage::new(source, format!("sensor/{}", sensor_type)),
            sensor_type,
            sensor_id,
            data,
            quality: 1.0,
            confidence: 1.0,
        }
    }
    
    pub fn with_quality(mut self, quality: f32) -> Self {
        self.quality = quality.clamp(0.0, 1.0);
        self
    }
    
    pub fn with_confidence(mut self, confidence: f32) -> Self {
        self.confidence = confidence.clamp(0.0, 1.0);
        self
    }
}

impl Message for SensorDataMessage {
    fn type_name(&self) -> &'static str {
        "SensorDataMessage"
    }
    
    fn id(&self) -> MessageId {
        self.base.id
    }
    
    fn timestamp(&self) -> crate::time::Time {
        self.base.timestamp
    }
    
    fn source(&self) -> &str {
        &self.base.source
    }
    
    fn topic(&self) -> &str {
        &self.base.topic
    }
    
    fn serialize(&self) -> anyhow::Result<Vec<u8>> {
        Ok(bincode::serialize(self)?)
    }
    
    fn clone_box(&self) -> Box<dyn Message> {
        Box::new(self.clone())
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// System status message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemStatusMessage {
    pub base: BaseMessage,
    pub node_name: String,
    pub status: SystemStatus,
    pub cpu_usage: f32,
    pub memory_usage: f64,
    pub disk_usage: f32,
    pub temperature: Option<f32>,
    pub battery_level: Option<f32>,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SystemStatus {
    Healthy,
    Warning,
    Error,
    Critical,
    Unknown,
}

impl SystemStatusMessage {
    pub fn new(source: String, node_name: String, status: SystemStatus) -> Self {
        Self {
            base: BaseMessage::new(source, "system/status".to_string()),
            node_name,
            status,
            cpu_usage: 0.0,
            memory_usage: 0.0,
            disk_usage: 0.0,
            temperature: None,
            battery_level: None,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }
    
    pub fn with_resources(mut self, cpu: f32, memory: f64, disk: f32) -> Self {
        self.cpu_usage = cpu.clamp(0.0, 100.0);
        self.memory_usage = memory.max(0.0);
        self.disk_usage = disk.clamp(0.0, 100.0);
        self
    }
    
    pub fn with_temperature(mut self, temp: f32) -> Self {
        self.temperature = Some(temp);
        self
    }
    
    pub fn with_battery(mut self, level: f32) -> Self {
        self.battery_level = Some(level.clamp(0.0, 100.0));
        self
    }
    
    pub fn add_error(mut self, error: String) -> Self {
        self.errors.push(error);
        self
    }
    
    pub fn add_warning(mut self, warning: String) -> Self {
        self.warnings.push(warning);
        self
    }
}

impl Message for SystemStatusMessage {
    fn type_name(&self) -> &'static str {
        "SystemStatusMessage"
    }
    
    fn id(&self) -> MessageId {
        self.base.id
    }
    
    fn timestamp(&self) -> crate::time::Time {
        self.base.timestamp
    }
    
    fn source(&self) -> &str {
        &self.base.source
    }
    
    fn topic(&self) -> &str {
        &self.base.topic
    }
    
    fn serialize(&self) -> anyhow::Result<Vec<u8>> {
        Ok(bincode::serialize(self)?)
    }
    
    fn clone_box(&self) -> Box<dyn Message> {
        Box::new(self.clone())
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Navigation goal message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavigationGoalMessage {
    pub base: BaseMessage,
    pub goal_id: String,
    pub target_pose: Pose3D,
    pub tolerance: f64,
    pub max_velocity: Option<Twist3D>,
    pub path_constraints: Vec<String>,
    pub timeout_ms: Option<u64>,
}

impl NavigationGoalMessage {
    pub fn new(source: String, goal_id: String, target_pose: Pose3D) -> Self {
        Self {
            base: BaseMessage::new(source, "navigation/goal".to_string()),
            goal_id,
            target_pose,
            tolerance: 0.1,
            max_velocity: None,
            path_constraints: Vec::new(),
            timeout_ms: None,
        }
    }
}

impl Message for NavigationGoalMessage {
    fn type_name(&self) -> &'static str {
        "NavigationGoalMessage"
    }
    
    fn id(&self) -> MessageId {
        self.base.id
    }
    
    fn timestamp(&self) -> crate::time::Time {
        self.base.timestamp
    }
    
    fn source(&self) -> &str {
        &self.base.source
    }
    
    fn topic(&self) -> &str {
        &self.base.topic
    }
    
    fn serialize(&self) -> anyhow::Result<Vec<u8>> {
        Ok(bincode::serialize(self)?)
    }
    
    fn clone_box(&self) -> Box<dyn Message> {
        Box::new(self.clone())
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}