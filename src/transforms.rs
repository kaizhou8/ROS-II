//! Transform system for coordinate frame management
//! 
//! Provides ROS-style TF (Transform) functionality for managing coordinate frames.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};
use anyhow::{Result, anyhow};
use crate::time::Time;

/// 3D Vector
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Vector3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vector3 {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }
    
    pub fn zero() -> Self {
        Self::new(0.0, 0.0, 0.0)
    }
    
    pub fn magnitude(&self) -> f64 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }
    
    pub fn normalize(&self) -> Self {
        let mag = self.magnitude();
        if mag > 0.0 {
            Self::new(self.x / mag, self.y / mag, self.z / mag)
        } else {
            *self
        }
    }
}

/// Quaternion for rotation representation
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Quaternion {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub w: f64,
}

impl Quaternion {
    pub fn new(x: f64, y: f64, z: f64, w: f64) -> Self {
        Self { x, y, z, w }
    }
    
    pub fn identity() -> Self {
        Self::new(0.0, 0.0, 0.0, 1.0)
    }
    
    pub fn from_euler(roll: f64, pitch: f64, yaw: f64) -> Self {
        let cr = (roll * 0.5).cos();
        let sr = (roll * 0.5).sin();
        let cp = (pitch * 0.5).cos();
        let sp = (pitch * 0.5).sin();
        let cy = (yaw * 0.5).cos();
        let sy = (yaw * 0.5).sin();
        
        Self::new(
            sr * cp * cy - cr * sp * sy,
            cr * sp * cy + sr * cp * sy,
            cr * cp * sy - sr * sp * cy,
            cr * cp * cy + sr * sp * sy,
        )
    }
    
    pub fn to_euler(&self) -> (f64, f64, f64) {
        let sinr_cosp = 2.0 * (self.w * self.x + self.y * self.z);
        let cosr_cosp = 1.0 - 2.0 * (self.x * self.x + self.y * self.y);
        let roll = sinr_cosp.atan2(cosr_cosp);
        
        let sinp = 2.0 * (self.w * self.y - self.z * self.x);
        let pitch = if sinp.abs() >= 1.0 {
            std::f64::consts::PI / 2.0 * sinp.signum()
        } else {
            sinp.asin()
        };
        
        let siny_cosp = 2.0 * (self.w * self.z + self.x * self.y);
        let cosy_cosp = 1.0 - 2.0 * (self.y * self.y + self.z * self.z);
        let yaw = siny_cosp.atan2(cosy_cosp);
        
        (roll, pitch, yaw)
    }
    
    pub fn normalize(&self) -> Self {
        let norm = (self.x * self.x + self.y * self.y + self.z * self.z + self.w * self.w).sqrt();
        if norm > 0.0 {
            Self::new(self.x / norm, self.y / norm, self.z / norm, self.w / norm)
        } else {
            Self::identity()
        }
    }
}

/// Transform between coordinate frames
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Transform {
    pub translation: Vector3,
    pub rotation: Quaternion,
}

impl Transform {
    pub fn new(translation: Vector3, rotation: Quaternion) -> Self {
        Self { translation, rotation }
    }
    
    pub fn identity() -> Self {
        Self::new(Vector3::zero(), Quaternion::identity())
    }
    
    pub fn from_translation(translation: Vector3) -> Self {
        Self::new(translation, Quaternion::identity())
    }
    
    pub fn from_rotation(rotation: Quaternion) -> Self {
        Self::new(Vector3::zero(), rotation)
    }
    
    pub fn inverse(&self) -> Self {
        let inv_rotation = Quaternion::new(-self.rotation.x, -self.rotation.y, -self.rotation.z, self.rotation.w);
        // Apply inverse rotation to translation
        let inv_translation = Vector3::new(
            -self.translation.x,
            -self.translation.y,
            -self.translation.z,
        );
        Self::new(inv_translation, inv_rotation)
    }
    
    pub fn apply_to_point(&self, point: Vector3) -> Vector3 {
        // Apply rotation then translation
        // Simplified rotation application (full quaternion rotation would be more complex)
        Vector3::new(
            point.x + self.translation.x,
            point.y + self.translation.y,
            point.z + self.translation.z,
        )
    }
}

/// Stamped transform with timestamp and frame information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StampedTransform {
    pub header: TransformHeader,
    pub transform: Transform,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TransformHeader {
    pub timestamp: Time,
    pub frame_id: String,
    pub child_frame_id: String,
}

impl StampedTransform {
    pub fn new(frame_id: String, child_frame_id: String, transform: Transform) -> Self {
        Self {
            header: TransformHeader {
                timestamp: Time::now(),
                frame_id,
                child_frame_id,
            },
            transform,
        }
    }
}

/// Transform buffer for storing and querying transforms
#[derive(Debug)]
pub struct TransformBuffer {
    transforms: Arc<RwLock<HashMap<String, HashMap<String, StampedTransform>>>>,
    max_cache_time: std::time::Duration,
}

impl TransformBuffer {
    pub fn new() -> Self {
        Self {
            transforms: Arc::new(RwLock::new(HashMap::new())),
            max_cache_time: std::time::Duration::from_secs(10),
        }
    }
    
    pub fn with_cache_time(cache_time: std::time::Duration) -> Self {
        Self {
            transforms: Arc::new(RwLock::new(HashMap::new())),
            max_cache_time: cache_time,
        }
    }
    
    pub async fn set_transform(&self, transform: StampedTransform) {
        let mut transforms = self.transforms.write().await;
        let parent_map = transforms.entry(transform.header.frame_id.clone()).or_insert_with(HashMap::new);
        parent_map.insert(transform.header.child_frame_id.clone(), transform);
    }
    
    pub async fn lookup_transform(&self, target_frame: &str, source_frame: &str) -> Result<Transform> {
        if target_frame == source_frame {
            return Ok(Transform::identity());
        }
        
        let transforms = self.transforms.read().await;
        
        // Direct transform lookup
        if let Some(parent_map) = transforms.get(target_frame) {
            if let Some(stamped_transform) = parent_map.get(source_frame) {
                return Ok(stamped_transform.transform.clone());
            }
        }
        
        // Inverse transform lookup
        if let Some(parent_map) = transforms.get(source_frame) {
            if let Some(stamped_transform) = parent_map.get(target_frame) {
                return Ok(stamped_transform.transform.inverse());
            }
        }
        
        // TODO: Implement chain lookup for indirect transforms
        Err(anyhow!("Transform from {} to {} not found", source_frame, target_frame))
    }
    
    pub async fn can_transform(&self, target_frame: &str, source_frame: &str) -> bool {
        self.lookup_transform(target_frame, source_frame).await.is_ok()
    }
    
    pub async fn get_frame_list(&self) -> Vec<String> {
        let transforms = self.transforms.read().await;
        let mut frames = std::collections::HashSet::new();
        
        for (parent, children) in transforms.iter() {
            frames.insert(parent.clone());
            for child in children.keys() {
                frames.insert(child.clone());
            }
        }
        
        frames.into_iter().collect()
    }
    
    pub async fn clear(&self) {
        let mut transforms = self.transforms.write().await;
        transforms.clear();
    }
    
    pub async fn clear_old_transforms(&self) {
        let now = Time::now();
        let mut transforms = self.transforms.write().await;
        
        transforms.retain(|_, children| {
            children.retain(|_, stamped_transform| {
                let age = now.duration_since(stamped_transform.header.timestamp);
                match age {
                    Ok(duration) => duration.to_std() < self.max_cache_time,
                    Err(_) => false, // Remove transforms that have invalid timestamps
                }
            });
            !children.is_empty()
        });
    }
}

impl Default for TransformBuffer {
    fn default() -> Self {
        Self::new()
    }
}

/// Transform broadcaster for publishing transforms
#[derive(Debug, Clone)]
pub struct TransformBroadcaster {
    buffer: Arc<TransformBuffer>,
}

impl TransformBroadcaster {
    pub fn new(buffer: Arc<TransformBuffer>) -> Self {
        Self { buffer }
    }
    
    pub async fn send_transform(&self, transform: StampedTransform) {
        self.buffer.set_transform(transform).await;
    }
    
    pub async fn send_transforms(&self, transforms: Vec<StampedTransform>) {
        for transform in transforms {
            self.buffer.set_transform(transform).await;
        }
    }
}

/// Transform listener for querying transforms
#[derive(Debug, Clone)]
pub struct TransformListener {
    buffer: Arc<TransformBuffer>,
}

impl TransformListener {
    pub fn new(buffer: Arc<TransformBuffer>) -> Self {
        Self { buffer }
    }
    
    pub async fn lookup_transform(&self, target_frame: &str, source_frame: &str) -> Result<Transform> {
        self.buffer.lookup_transform(target_frame, source_frame).await
    }
    
    pub async fn can_transform(&self, target_frame: &str, source_frame: &str) -> bool {
        self.buffer.can_transform(target_frame, source_frame).await
    }
    
    pub async fn wait_for_transform(&self, target_frame: &str, source_frame: &str, timeout: std::time::Duration) -> Result<Transform> {
        let start = std::time::Instant::now();
        
        while start.elapsed() < timeout {
            if let Ok(transform) = self.lookup_transform(target_frame, source_frame).await {
                return Ok(transform);
            }
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
        
        Err(anyhow!("Timeout waiting for transform from {} to {}", source_frame, target_frame))
    }
    
    pub async fn get_frame_list(&self) -> Vec<String> {
        self.buffer.get_frame_list().await
    }
}