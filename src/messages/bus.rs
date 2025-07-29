//! Message bus implementation
//! 
//! High-performance message routing and delivery system.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use anyhow::{Result, anyhow};

use super::{Message, Topic, Subscription, Publisher};

/// Central message bus for routing messages between nodes
#[derive(Debug, Clone)]
pub struct MessageBus {
    topics: Arc<RwLock<HashMap<Topic, broadcast::Sender<Arc<dyn Message>>>>>,
    buffer_size: usize,
}

impl MessageBus {
    /// Create a new message bus
    pub async fn new(buffer_size: usize) -> Result<Self> {
        Ok(Self {
            topics: Arc::new(RwLock::new(HashMap::new())),
            buffer_size,
        })
    }
    
    /// Create a publisher for a topic
    pub async fn create_publisher(&self, topic: impl Into<Topic>) -> Result<Publisher> {
        let topic = topic.into();
        let mut topics = self.topics.write().await;
        
        let sender = if let Some(existing_sender) = topics.get(&topic) {
            existing_sender.clone()
        } else {
            let (sender, _) = broadcast::channel(self.buffer_size);
            topics.insert(topic.clone(), sender.clone());
            sender
        };
        
        Ok(Publisher::new(topic, sender))
    }
    
    /// Create a subscription to a topic
    pub async fn subscribe(&self, topic: impl Into<Topic>) -> Result<Subscription> {
        let topic = topic.into();
        let mut topics = self.topics.write().await;
        
        let sender = if let Some(existing_sender) = topics.get(&topic) {
            existing_sender.clone()
        } else {
            let (sender, _) = broadcast::channel(self.buffer_size);
            topics.insert(topic.clone(), sender.clone());
            sender
        };
        
        let receiver = sender.subscribe();
        Ok(Subscription::new(topic, receiver))
    }
    
    /// Publish a message to a topic
    pub async fn publish(&self, topic: impl Into<Topic>, message: impl Message + 'static) -> Result<()> {
        let topic = topic.into();
        let topics = self.topics.read().await;
        
        if let Some(sender) = topics.get(&topic) {
            let message_arc = Arc::new(message) as Arc<dyn Message>;
            sender.send(message_arc)?;
        } else {
            return Err(anyhow!("Topic '{}' does not exist", topic));
        }
        
        Ok(())
    }
    
    /// Get all active topics
    pub async fn list_topics(&self) -> Vec<Topic> {
        let topics = self.topics.read().await;
        topics.keys().cloned().collect()
    }
    
    /// Get the number of subscribers for a topic
    pub async fn subscriber_count(&self, topic: &Topic) -> usize {
        let topics = self.topics.read().await;
        if let Some(sender) = topics.get(topic) {
            sender.receiver_count()
        } else {
            0
        }
    }
    
    /// Remove a topic and all its subscribers
    pub async fn remove_topic(&self, topic: &Topic) -> Result<()> {
        let mut topics = self.topics.write().await;
        topics.remove(topic);
        log::debug!("Removed topic: {}", topic);
        Ok(())
    }
    
    /// Get message bus statistics
    pub async fn get_stats(&self) -> MessageBusStats {
        let topics = self.topics.read().await;
        let mut stats = MessageBusStats {
            total_topics: topics.len(),
            total_subscribers: 0,
            topics_info: Vec::new(),
        };
        
        for (topic, sender) in topics.iter() {
            let subscriber_count = sender.receiver_count();
            stats.total_subscribers += subscriber_count;
            stats.topics_info.push(TopicInfo {
                name: topic.clone(),
                subscriber_count,
                buffer_size: self.buffer_size,
            });
        }
        
        stats
    }
    
    /// Clear all topics and subscribers
    pub async fn clear(&self) -> Result<()> {
        let mut topics = self.topics.write().await;
        topics.clear();
        log::info!("Cleared all topics from message bus");
        Ok(())
    }
}

/// Message bus statistics
#[derive(Debug, Clone)]
pub struct MessageBusStats {
    pub total_topics: usize,
    pub total_subscribers: usize,
    pub topics_info: Vec<TopicInfo>,
}

/// Information about a specific topic
#[derive(Debug, Clone)]
pub struct TopicInfo {
    pub name: Topic,
    pub subscriber_count: usize,
    pub buffer_size: usize,
}