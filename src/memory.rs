//! Memory management utilities for Robot Framework Rust

use crate::{Result, RobotError};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};

/// Memory pool for efficient allocation
pub struct MemoryPool<T> {
    pool: Vec<T>,
    capacity: usize,
}

impl<T: Default> MemoryPool<T> {
    pub fn new(capacity: usize) -> Self {
        Self {
            pool: Vec::with_capacity(capacity),
            capacity,
        }
    }

    pub fn get(&mut self) -> T {
        self.pool.pop().unwrap_or_default()
    }

    pub fn put(&mut self, item: T) {
        if self.pool.len() < self.capacity {
            self.pool.push(item);
        }
    }
}

/// Ring buffer for circular data storage
pub struct RingBuffer<T> {
    buffer: Vec<Option<T>>,
    head: usize,
    tail: usize,
    size: usize,
    capacity: usize,
}

impl<T> RingBuffer<T> {
    pub fn new(capacity: usize) -> Self {
        let mut buffer = Vec::with_capacity(capacity);
        for _ in 0..capacity {
            buffer.push(None);
        }
        
        Self {
            buffer,
            head: 0,
            tail: 0,
            size: 0,
            capacity,
        }
    }

    pub fn push(&mut self, item: T) -> Option<T> {
        let old_item = self.buffer[self.tail].take();
        self.buffer[self.tail] = Some(item);
        self.tail = (self.tail + 1) % self.capacity;
        
        if self.size < self.capacity {
            self.size += 1;
        } else {
            self.head = (self.head + 1) % self.capacity;
        }
        
        old_item
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.size == 0 {
            return None;
        }
        
        let item = self.buffer[self.head].take();
        self.head = (self.head + 1) % self.capacity;
        self.size -= 1;
        item
    }

    pub fn len(&self) -> usize {
        self.size
    }

    pub fn is_empty(&self) -> bool {
        self.size == 0
    }
}

/// Memory usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryUsage {
    pub total_allocated: usize,
    pub current_usage: usize,
    pub peak_usage: usize,
    pub allocation_count: u64,
    pub deallocation_count: u64,
}

impl Default for MemoryUsage {
    fn default() -> Self {
        Self {
            total_allocated: 0,
            current_usage: 0,
            peak_usage: 0,
            allocation_count: 0,
            deallocation_count: 0,
        }
    }
}

/// Memory monitor for tracking allocations
pub struct MemoryMonitor {
    usage: Arc<Mutex<MemoryUsage>>,
}

impl MemoryMonitor {
    pub fn new() -> Self {
        Self {
            usage: Arc::new(Mutex::new(MemoryUsage::default())),
        }
    }

    pub fn record_allocation(&self, size: usize) -> Result<()> {
        let mut usage = self.usage.lock()
            .map_err(|e| RobotError::Memory(format!("Failed to lock usage: {}", e)))?;
        
        usage.total_allocated += size;
        usage.current_usage += size;
        usage.allocation_count += 1;
        
        if usage.current_usage > usage.peak_usage {
            usage.peak_usage = usage.current_usage;
        }
        
        Ok(())
    }

    pub fn record_deallocation(&self, size: usize) -> Result<()> {
        let mut usage = self.usage.lock()
            .map_err(|e| RobotError::Memory(format!("Failed to lock usage: {}", e)))?;
        
        usage.current_usage = usage.current_usage.saturating_sub(size);
        usage.deallocation_count += 1;
        
        Ok(())
    }

    pub fn get_usage(&self) -> Result<MemoryUsage> {
        let usage = self.usage.lock()
            .map_err(|e| RobotError::Memory(format!("Failed to lock usage: {}", e)))?;
        
        Ok(usage.clone())
    }
}

/// Tracked box for memory monitoring
pub struct TrackedBox<T> {
    inner: Box<T>,
    size: usize,
    monitor: Arc<MemoryMonitor>,
}

impl<T> TrackedBox<T> {
    pub fn new(value: T, monitor: Arc<MemoryMonitor>) -> Result<Self> {
        let size = std::mem::size_of::<T>();
        monitor.record_allocation(size)?;
        
        Ok(Self {
            inner: Box::new(value),
            size,
            monitor,
        })
    }
}

impl<T> Drop for TrackedBox<T> {
    fn drop(&mut self) {
        let _ = self.monitor.record_deallocation(self.size);
    }
}

impl<T> std::ops::Deref for TrackedBox<T> {
    type Target = T;
    
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> std::ops::DerefMut for TrackedBox<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

/// String interner for memory efficiency
pub struct StringInterner {
    strings: HashMap<String, u32>,
    reverse: HashMap<u32, String>,
    next_id: u32,
}

impl StringInterner {
    pub fn new() -> Self {
        Self {
            strings: HashMap::new(),
            reverse: HashMap::new(),
            next_id: 0,
        }
    }

    pub fn intern(&mut self, s: &str) -> u32 {
        if let Some(&id) = self.strings.get(s) {
            id
        } else {
            let id = self.next_id;
            self.next_id += 1;
            self.strings.insert(s.to_string(), id);
            self.reverse.insert(id, s.to_string());
            id
        }
    }

    pub fn get(&self, id: u32) -> Option<&String> {
        self.reverse.get(&id)
    }
}