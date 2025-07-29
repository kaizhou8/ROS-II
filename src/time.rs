//! Time utilities for the robot framework
//! 
//! Provides time-related functionality with high precision and ROS compatibility.

use std::ops::{Add, Sub};
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

/// High-precision timestamp
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Time {
    /// Seconds since Unix epoch
    pub sec: i64,
    /// Nanoseconds within the second
    pub nsec: u32,
}

impl Time {
    /// Create a new time from seconds and nanoseconds
    pub fn new(sec: i64, nsec: u32) -> Self {
        Self { sec, nsec }
    }
    
    /// Get the current time
    pub fn now() -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default();
        
        Self {
            sec: now.as_secs() as i64,
            nsec: now.subsec_nanos(),
        }
    }
    
    /// Create time from standard Duration
    pub fn from_duration(duration: std::time::Duration) -> Self {
        Self {
            sec: duration.as_secs() as i64,
            nsec: duration.subsec_nanos(),
        }
    }
    
    /// Convert to standard Duration
    pub fn to_duration(&self) -> std::time::Duration {
        std::time::Duration::new(self.sec as u64, self.nsec)
    }
    
    /// Convert to chrono DateTime
    pub fn to_datetime(&self) -> DateTime<Utc> {
        DateTime::from_timestamp(self.sec, self.nsec).unwrap_or_default()
    }
    
    /// Create from chrono DateTime
    pub fn from_datetime(dt: DateTime<Utc>) -> Self {
        Self {
            sec: dt.timestamp(),
            nsec: dt.timestamp_subsec_nanos(),
        }
    }
    
    /// Get time as floating point seconds
    pub fn as_secs_f64(&self) -> f64 {
        self.sec as f64 + (self.nsec as f64 / 1_000_000_000.0)
    }
    
    /// Create from floating point seconds
    pub fn from_secs_f64(secs: f64) -> Self {
        let sec = secs.floor() as i64;
        let nsec = ((secs - sec as f64) * 1_000_000_000.0) as u32;
        Self { sec, nsec }
    }
    
    /// Calculate duration since another time
    pub fn duration_since(&self, earlier: Time) -> Result<Duration, std::time::SystemTimeError> {
        if *self >= earlier {
            Ok(Duration::new(
                (self.sec - earlier.sec) as u64,
                if self.nsec >= earlier.nsec {
                    self.nsec - earlier.nsec
                } else {
                    1_000_000_000 + self.nsec - earlier.nsec
                }
            ))
        } else {
            // Create a SystemTimeError using from_raw_os_error or similar approach
            // Since SystemTimeError doesn't have new(), we need to create it differently
            use std::io::{Error, ErrorKind};
            let _io_error = Error::new(ErrorKind::InvalidInput, "Time is earlier than reference");
            // Convert to SystemTimeError by using a Duration calculation that would fail
            Err(std::time::UNIX_EPOCH.duration_since(std::time::SystemTime::now()).unwrap_err())
        }
    }
    
    /// Convert to std::time::Duration for compatibility
    pub fn as_std_duration(&self) -> Result<std::time::Duration, std::time::SystemTimeError> {
        Ok(std::time::Duration::new(self.sec as u64, self.nsec))
    }
    
    /// Add a duration to this time
    pub fn add_duration(&self, duration: Duration) -> Self {
        let total_nsec = self.nsec as u64 + duration.nsec as u64;
        let extra_sec = total_nsec / 1_000_000_000;
        let nsec = (total_nsec % 1_000_000_000) as u32;
        
        Self {
            sec: self.sec + duration.sec as i64 + extra_sec as i64,
            nsec,
        }
    }
    
    /// Subtract a duration from this time
    pub fn sub_duration(&self, duration: Duration) -> Self {
        let total_nsec = if self.nsec >= duration.nsec {
            self.nsec - duration.nsec
        } else {
            1_000_000_000 + self.nsec - duration.nsec
        };
        
        let sec_adjustment = if self.nsec < duration.nsec { 1 } else { 0 };
        
        Self {
            sec: self.sec - duration.sec as i64 - sec_adjustment,
            nsec: total_nsec,
        }
    }
}

impl Default for Time {
    fn default() -> Self {
        Self::now()
    }
}

impl std::fmt::Display for Time {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{:09}", self.sec, self.nsec)
    }
}

/// Duration type compatible with ROS
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Duration {
    /// Seconds component
    pub sec: u64,
    /// Nanoseconds component
    pub nsec: u32,
}

impl Duration {
    /// Create a new duration
    pub fn new(sec: u64, nsec: u32) -> Self {
        Self { sec, nsec }
    }
    
    /// Create duration from seconds
    pub fn from_secs(secs: u64) -> Self {
        Self { sec: secs, nsec: 0 }
    }
    
    /// Create duration from milliseconds
    pub fn from_millis(millis: u64) -> Self {
        Self {
            sec: millis / 1000,
            nsec: ((millis % 1000) * 1_000_000) as u32,
        }
    }
    
    /// Create duration from microseconds
    pub fn from_micros(micros: u64) -> Self {
        Self {
            sec: micros / 1_000_000,
            nsec: ((micros % 1_000_000) * 1000) as u32,
        }
    }
    
    /// Create duration from nanoseconds
    pub fn from_nanos(nanos: u64) -> Self {
        Self {
            sec: nanos / 1_000_000_000,
            nsec: (nanos % 1_000_000_000) as u32,
        }
    }
    
    /// Create duration from floating point seconds
    pub fn from_secs_f64(secs: f64) -> Self {
        let sec = secs.floor() as u64;
        let nsec = ((secs - sec as f64) * 1_000_000_000.0) as u32;
        Self { sec, nsec }
    }
    
    /// Convert to standard Duration
    pub fn to_std(&self) -> std::time::Duration {
        std::time::Duration::new(self.sec, self.nsec)
    }
    
    /// Convert to standard Duration (alias for to_std)
    pub fn as_std_duration(&self) -> Result<std::time::Duration, std::time::SystemTimeError> {
        Ok(self.to_std())
    }
    
    /// Create from standard Duration
    pub fn from_std(duration: std::time::Duration) -> Self {
        Self {
            sec: duration.as_secs(),
            nsec: duration.subsec_nanos(),
        }
    }
    
    /// Get duration as floating point seconds
    pub fn as_secs_f64(&self) -> f64 {
        self.sec as f64 + (self.nsec as f64 / 1_000_000_000.0)
    }
    
    /// Get duration in milliseconds
    pub fn as_millis(&self) -> u64 {
        self.sec * 1000 + (self.nsec / 1_000_000) as u64
    }
    
    /// Get duration in microseconds
    pub fn as_micros(&self) -> u64 {
        self.sec * 1_000_000 + (self.nsec / 1000) as u64
    }
    
    /// Get duration in nanoseconds
    pub fn as_nanos(&self) -> u64 {
        self.sec * 1_000_000_000 + self.nsec as u64
    }
    
    /// Check if duration is zero
    pub fn is_zero(&self) -> bool {
        self.sec == 0 && self.nsec == 0
    }
}

impl Default for Duration {
    fn default() -> Self {
        Self { sec: 0, nsec: 0 }
    }
}

impl std::fmt::Display for Duration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{:09}s", self.sec, self.nsec)
    }
}

impl Add for Duration {
    type Output = Duration;
    
    fn add(self, other: Duration) -> Duration {
        let total_nsec = self.nsec as u64 + other.nsec as u64;
        let extra_sec = total_nsec / 1_000_000_000;
        let nsec = (total_nsec % 1_000_000_000) as u32;
        
        Duration {
            sec: self.sec + other.sec + extra_sec,
            nsec,
        }
    }
}

impl Sub for Duration {
    type Output = Duration;
    
    fn sub(self, other: Duration) -> Duration {
        if self < other {
            return Duration::default();
        }
        
        let (sec, nsec) = if self.nsec >= other.nsec {
            (self.sec - other.sec, self.nsec - other.nsec)
        } else {
            (self.sec - other.sec - 1, 1_000_000_000 + self.nsec - other.nsec)
        };
        
        Duration { sec, nsec }
    }
}

impl Add<Duration> for Time {
    type Output = Time;
    
    fn add(self, duration: Duration) -> Time {
        self.add_duration(duration)
    }
}

impl Sub<Duration> for Time {
    type Output = Time;
    
    fn sub(self, duration: Duration) -> Time {
        self.sub_duration(duration)
    }
}

impl Sub for Time {
    type Output = Duration;
    
    fn sub(self, other: Time) -> Duration {
        self.duration_since(other).unwrap_or_default()
    }
}

/// Rate limiter for controlling execution frequency
#[derive(Debug)]
pub struct Rate {
    period: Duration,
    last_time: Option<Time>,
}

impl Rate {
    /// Create a new rate with the given frequency in Hz
    pub fn new(frequency: f64) -> Self {
        let period = Duration::from_secs_f64(1.0 / frequency);
        Self {
            period,
            last_time: None,
        }
    }
    
    /// Create a new rate with the given period
    pub fn with_period(period: Duration) -> Self {
        Self {
            period,
            last_time: None,
        }
    }
    
    /// Sleep until the next period
    pub async fn sleep(&mut self) {
        let now = Time::now();
        
        if let Some(last_time) = self.last_time {
            let elapsed = now.duration_since(last_time).unwrap_or_default();
            if elapsed < self.period {
                let sleep_duration = self.period - elapsed;
                tokio::time::sleep(sleep_duration.to_std()).await;
            }
        }
        
        self.last_time = Some(Time::now());
    }
    
    /// Get the expected period
    pub fn period(&self) -> Duration {
        self.period
    }
    
    /// Get the frequency in Hz
    pub fn frequency(&self) -> f64 {
        1.0 / (self.period.sec as f64 + self.period.nsec as f64 / 1_000_000_000.0)
    }
    
    /// Reset the rate timer
    pub fn reset(&mut self) {
        self.last_time = None;
    }
}