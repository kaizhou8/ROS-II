//! Debug module for Robot Framework Rust
//! 
//! Provides comprehensive debugging and profiling tools for robotics applications.

use crate::{Result, RobotError};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};

pub mod debugger;
pub mod profiler;
pub mod tracer;
pub mod analyzer;

pub use debugger::Debugger;
pub use profiler::Profiler;
pub use tracer::TraceCollector;
pub use analyzer::PerformanceAnalyzer;

/// Debug configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugConfig {
    /// Enable performance profiling
    pub enable_profiling: bool,
    /// Enable trace collection
    pub enable_tracing: bool,
    /// Enable memory debugging
    pub enable_memory_debug: bool,
    /// Maximum number of traces to keep
    pub max_traces: usize,
    /// Profiling sample rate (0.0 to 1.0)
    pub sample_rate: f64,
    /// Enable real-time debugging
    pub real_time: bool,
}

impl Default for DebugConfig {
    fn default() -> Self {
        Self {
            enable_profiling: true,
            enable_tracing: true,
            enable_memory_debug: true,
            max_traces: 10000,
            sample_rate: 0.1,
            real_time: true,
        }
    }
}

/// Debug event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DebugEvent {
    FunctionEnter { name: String, timestamp: u64, thread_id: u64 },
    FunctionExit { name: String, timestamp: u64, thread_id: u64, duration: u64 },
    MemoryAllocation { size: usize, timestamp: u64, location: String },
    MemoryDeallocation { size: usize, timestamp: u64, location: String },
    Error { message: String, timestamp: u64, stack_trace: Vec<String> },
    Warning { message: String, timestamp: u64, location: String },
    Breakpoint { id: String, location: String, timestamp: u64 },
    VariableChanged { name: String, old_value: String, new_value: String, timestamp: u64 },
}

/// Performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub function_name: String,
    pub call_count: u64,
    pub total_duration: Duration,
    pub average_duration: Duration,
    pub min_duration: Duration,
    pub max_duration: Duration,
    pub memory_usage: usize,
}

/// Debug manager
pub struct DebugManager {
    config: DebugConfig,
    debugger: Option<Debugger>,
    profiler: Option<Profiler>,
    trace_collector: Option<TraceCollector>,
    performance_analyzer: Option<PerformanceAnalyzer>,
    events: Arc<Mutex<Vec<DebugEvent>>>,
    metrics: Arc<Mutex<HashMap<String, PerformanceMetrics>>>,
}

impl DebugManager {
    /// Create a new debug manager
    pub fn new(config: DebugConfig) -> Self {
        Self {
            config,
            debugger: None,
            profiler: None,
            trace_collector: None,
            performance_analyzer: None,
            events: Arc::new(Mutex::new(Vec::new())),
            metrics: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Initialize the debug system
    pub async fn init(&mut self) -> Result<()> {
        log::info!("Initializing debug system");

        // Initialize debugger
        let debugger = Debugger::new(Arc::clone(&self.events)).await?;
        self.debugger = Some(debugger);

        // Initialize profiler
        if self.config.enable_profiling {
            let profiler = Profiler::new(
                Arc::clone(&self.events),
                Arc::clone(&self.metrics),
                self.config.sample_rate,
            ).await?;
            self.profiler = Some(profiler);
        }

        // Initialize trace collector
        if self.config.enable_tracing {
            let trace_collector = TraceCollector::new(
                Arc::clone(&self.events),
                self.config.max_traces,
            ).await?;
            self.trace_collector = Some(trace_collector);
        }

        // Initialize performance analyzer
        let performance_analyzer = PerformanceAnalyzer::new(
            Arc::clone(&self.events),
            Arc::clone(&self.metrics),
        ).await?;
        self.performance_analyzer = Some(performance_analyzer);

        log::info!("Debug system initialized successfully");
        Ok(())
    }

    /// Start debugging
    pub async fn start(&mut self) -> Result<()> {
        log::info!("Starting debug system");

        if let Some(debugger) = &mut self.debugger {
            debugger.start().await?;
        }

        if let Some(profiler) = &mut self.profiler {
            profiler.start().await?;
        }

        if let Some(trace_collector) = &mut self.trace_collector {
            trace_collector.start().await?;
        }

        if let Some(performance_analyzer) = &mut self.performance_analyzer {
            performance_analyzer.start().await?;
        }

        log::info!("Debug system started successfully");
        Ok(())
    }

    /// Stop debugging
    pub async fn stop(&mut self) -> Result<()> {
        log::info!("Stopping debug system");

        if let Some(performance_analyzer) = &mut self.performance_analyzer {
            performance_analyzer.stop().await?;
        }

        if let Some(trace_collector) = &mut self.trace_collector {
            trace_collector.stop().await?;
        }

        if let Some(profiler) = &mut self.profiler {
            profiler.stop().await?;
        }

        if let Some(debugger) = &mut self.debugger {
            debugger.stop().await?;
        }

        log::info!("Debug system stopped successfully");
        Ok(())
    }

    /// Add a debug event
    pub fn add_event(&self, event: DebugEvent) -> Result<()> {
        let mut events = self.events.lock()
            .map_err(|e| RobotError::Debug(format!("Failed to lock events: {}", e)))?;
        
        events.push(event);

        // Limit events
        if events.len() > self.config.max_traces {
            events.remove(0);
        }

        Ok(())
    }

    /// Get debug events
    pub fn get_events(&self) -> Result<Vec<DebugEvent>> {
        let events = self.events.lock()
            .map_err(|e| RobotError::Debug(format!("Failed to lock events: {}", e)))?;
        
        Ok(events.clone())
    }

    /// Get performance metrics
    pub fn get_metrics(&self) -> Result<HashMap<String, PerformanceMetrics>> {
        let metrics = self.metrics.lock()
            .map_err(|e| RobotError::Debug(format!("Failed to lock metrics: {}", e)))?;
        
        Ok(metrics.clone())
    }

    /// Set breakpoint
    pub fn set_breakpoint(&mut self, id: &str, location: &str) -> Result<()> {
        if let Some(debugger) = &mut self.debugger {
            debugger.set_breakpoint(id, location)?;
        }
        Ok(())
    }

    /// Remove breakpoint
    pub fn remove_breakpoint(&mut self, id: &str) -> Result<()> {
        if let Some(debugger) = &mut self.debugger {
            debugger.remove_breakpoint(id)?;
        }
        Ok(())
    }

    /// Generate performance report
    pub fn generate_report(&self) -> Result<String> {
        let metrics = self.get_metrics()?;
        let events = self.get_events()?;

        let mut report = String::new();
        report.push_str("# Robot Framework Rust Debug Report\n\n");

        // Performance metrics
        report.push_str("## Performance Metrics\n\n");
        for (name, metric) in &metrics {
            report.push_str(&format!(
                "### {}\n\
                - Call Count: {}\n\
                - Total Duration: {:?}\n\
                - Average Duration: {:?}\n\
                - Min Duration: {:?}\n\
                - Max Duration: {:?}\n\
                - Memory Usage: {} bytes\n\n",
                name,
                metric.call_count,
                metric.total_duration,
                metric.average_duration,
                metric.min_duration,
                metric.max_duration,
                metric.memory_usage
            ));
        }

        // Event summary
        report.push_str("## Event Summary\n\n");
        let mut event_counts = HashMap::new();
        for event in &events {
            let event_type = match event {
                DebugEvent::FunctionEnter { .. } => "Function Enter",
                DebugEvent::FunctionExit { .. } => "Function Exit",
                DebugEvent::MemoryAllocation { .. } => "Memory Allocation",
                DebugEvent::MemoryDeallocation { .. } => "Memory Deallocation",
                DebugEvent::Error { .. } => "Error",
                DebugEvent::Warning { .. } => "Warning",
                DebugEvent::Breakpoint { .. } => "Breakpoint",
                DebugEvent::VariableChanged { .. } => "Variable Changed",
            };
            *event_counts.entry(event_type).or_insert(0) += 1;
        }

        for (event_type, count) in event_counts {
            report.push_str(&format!("- {}: {}\n", event_type, count));
        }

        Ok(report)
    }
}

/// Global debug manager instance
static mut DEBUG_MANAGER: Option<DebugManager> = None;
static INIT: std::sync::Once = std::sync::Once::new();

/// Initialize the debugger
pub fn init_debugger() -> Result<()> {
    INIT.call_once(|| {
        let config = DebugConfig::default();
        let manager = DebugManager::new(config);
        unsafe {
            DEBUG_MANAGER = Some(manager);
        }
    });
    Ok(())
}

/// Get the global debug manager
pub fn get_manager() -> Result<&'static mut DebugManager> {
    unsafe {
        DEBUG_MANAGER.as_mut()
            .ok_or_else(|| RobotError::Debug("Debug system not initialized".to_string()))
    }
}

/// Add a debug event
pub fn add_event(event: DebugEvent) -> Result<()> {
    get_manager()?.add_event(event)
}

/// Set a breakpoint
pub fn set_breakpoint(id: &str, location: &str) -> Result<()> {
    get_manager()?.set_breakpoint(id, location)
}

/// Remove a breakpoint
pub fn remove_breakpoint(id: &str) -> Result<()> {
    get_manager()?.remove_breakpoint(id)
}

/// Generate debug report
pub fn generate_report() -> Result<String> {
    get_manager()?.generate_report()
}

/// Debug macro for function tracing
#[macro_export]
macro_rules! debug_trace {
    ($func_name:expr) => {
        let _guard = $crate::debug::FunctionTraceGuard::new($func_name);
    };
}

/// Function trace guard for automatic enter/exit tracking
pub struct FunctionTraceGuard {
    name: String,
    start_time: Instant,
}

impl FunctionTraceGuard {
    pub fn new(name: &str) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        
        let thread_id = std::thread::current().id().as_u64().get();
        
        let _ = add_event(DebugEvent::FunctionEnter {
            name: name.to_string(),
            timestamp: now,
            thread_id,
        });

        Self {
            name: name.to_string(),
            start_time: Instant::now(),
        }
    }
}

impl Drop for FunctionTraceGuard {
    fn drop(&mut self) {
        let duration = self.start_time.elapsed().as_micros() as u64;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        
        let thread_id = std::thread::current().id().as_u64().get();
        
        let _ = add_event(DebugEvent::FunctionExit {
            name: self.name.clone(),
            timestamp: now,
            thread_id,
            duration,
        });
    }
}