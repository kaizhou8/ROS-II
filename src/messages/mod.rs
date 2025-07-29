//! Message system for inter-node communication
//! 
//! Provides a high-performance, type-safe message passing system.

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use anyhow::Result;

pub mod bus;
pub mod types;

pub use bus::MessageBus;
pub use types::*;

/// Unique identifier for messages
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MessageId(pub Uuid);

impl MessageId {
    /// Create a new random message ID
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for MessageId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for MessageId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Core message trait that all messages must implement
pub trait Message: Send + Sync + Debug + Any {
    /// Get the message type name
    fn type_name(&self) -> &'static str;
    
    /// Get the message ID
    fn id(&self) -> MessageId;
    
    /// Get the timestamp when the message was created
    fn timestamp(&self) -> crate::time::Time;
    
    /// Get the source node name
    fn source(&self) -> &str;
    
    /// Get the topic this message belongs to
    fn topic(&self) -> &str;
    
    /// Serialize the message to bytes
    fn serialize(&self) -> Result<Vec<u8>>;
    
    /// Clone the message as a boxed trait object
    fn clone_box(&self) -> Box<dyn Message>;
    
    /// Downcast to concrete type
    fn as_any(&self) -> &dyn Any;
}

/// Base message structure providing common fields
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseMessage {
    pub id: MessageId,
    pub timestamp: crate::time::Time,
    pub source: String,
    pub topic: String,
}

impl BaseMessage {
    /// Create a new base message
    pub fn new(source: String, topic: String) -> Self {
        Self {
            id: MessageId::new(),
            timestamp: crate::time::Time::now(),
            source,
            topic,
        }
    }
}

/// Topic name type
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Topic(pub String);

impl Topic {
    /// Create a new topic
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }
    
    /// Check if this topic matches a pattern (supports wildcards)
    pub fn matches(&self, pattern: &str) -> bool {
        if pattern == "*" {
            return true;
        }
        
        if pattern.contains('*') {
            // Simple wildcard matching
            let parts: Vec<&str> = pattern.split('*').collect();
            if parts.len() == 2 {
                let prefix = parts[0];
                let suffix = parts[1];
                return self.0.starts_with(prefix) && self.0.ends_with(suffix);
            }
        }
        
        self.0 == pattern
    }
}

impl std::fmt::Display for Topic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for Topic {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for Topic {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

/// Message subscription handle
#[derive(Debug)]
pub struct Subscription {
    topic: Topic,
    receiver: broadcast::Receiver<Arc<dyn Message>>,
}

impl Subscription {
    /// Create a new subscription
    pub fn new(topic: Topic, receiver: broadcast::Receiver<Arc<dyn Message>>) -> Self {
        Self { topic, receiver }
    }
    
    /// Get the topic this subscription is for
    pub fn topic(&self) -> &Topic {
        &self.topic
    }
    
    /// Receive the next message
    pub async fn recv(&mut self) -> Result<Arc<dyn Message>> {
        Ok(self.receiver.recv().await?)
    }
    
    /// Try to receive a message without blocking
    pub fn try_recv(&mut self) -> Result<Arc<dyn Message>> {
        Ok(self.receiver.try_recv()?)
    }
}

/// Publisher handle for sending messages
#[derive(Debug, Clone)]
pub struct Publisher {
    topic: Topic,
    sender: broadcast::Sender<Arc<dyn Message>>,
}

impl Publisher {
    /// Create a new publisher
    pub fn new(topic: Topic, sender: broadcast::Sender<Arc<dyn Message>>) -> Self {
        Self { topic, sender }
    }
    
    /// Get the topic this publisher sends to
    pub fn topic(&self) -> &Topic {
        &self.topic
    }
    
    /// Publish a message
    pub async fn publish(&self, message: impl Message + 'static) -> Result<()> {
        let message_arc = Arc::new(message) as Arc<dyn Message>;
        self.sender.send(message_arc)?;
        Ok(())
    }
    
    /// Get the number of active subscribers
    pub fn subscriber_count(&self) -> usize {
        self.sender.receiver_count()
    }
}