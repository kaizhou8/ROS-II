# Robot Framework Rust

A high-performance, memory-safe robotics framework built with Rust, providing real-time capabilities, cross-platform support, and full ROS compatibility.

## 🚀 Features

- **Zero-cost abstractions**: High performance through Rust's type system
- **Memory safety**: Safety guarantees without runtime overhead
- **Real-time capabilities**: Deterministic performance without garbage collection
- **Cross-platform**: From microcontrollers to servers
- **Async-first**: Efficient concurrency based on tokio
- **ROS compatible**: Complete ROS-style communication patterns
  - **Messaging**: Publish/subscribe pattern
  - **Services**: Request/response communication
  - **Actions**: Long-running task management
  - **Transforms**: TF system for coordinate frame management

## 📋 System Requirements

- Rust 1.70+ 
- Supported platforms:
  - Windows (x86_64, ARM64)
  - Linux (x86_64, ARM64, ARM)
  - macOS (x86_64, ARM64)
  - Embedded platforms (ARM Cortex-M, ESP32, RISC-V)

## 🛠️ Quick Start

### Installation

```bash
# Clone repository
git clone https://github.com/your-org/robot-framework-rust.git
cd robot-framework-rust

# Build project
cargo build --release

# Run example
cargo run --example simple_robot
```

### Basic Usage

```rust
use robot_framework_rust::*;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize robot system
    let system = init().await?;
    
    // Create sensor node
    let sensor = SensorNode::new(
        "my_sensor".to_string(),
        "/sensors".to_string(),
        "temperature".to_string(),
        "temp_01".to_string(),
        10.0, // 10 Hz
        Box::new(RandomDataGenerator::new("temperature".to_string())),
    );
    
    // Add node to system
    let node_id = system.add_node(Box::new(sensor)).await?;
    
    // Start system
    system.start().await?;
    
    // Run for some time
    tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
    
    // Graceful shutdown
    system.stop().await?;
    
    Ok(())
}
```

## 🏗️ Architecture

### Core Components

- **Node System**: Modular node architecture based on traits
- **Message System**: High-performance publish-subscribe messaging
- **Time System**: High-precision timestamps and rate control
- **Configuration System**: Flexible configuration management and parameter server
- **Logging System**: Structured logging

### Message Types

The framework provides common robot message types:

- `RobotActionMessage`: Robot action commands
- `SensorDataMessage`: Sensor data
- `SystemStatusMessage`: System status information
- `NavigationGoalMessage`: Navigation goals

## 📊 Performance Characteristics

| Feature | Rust Version | C# Version |
|------|-----------|---------|
| Memory Usage | ~10-50MB | ~200-400MB |
| Startup Time | <100ms | ~2-5s |
| Binary Size | ~5-20MB | ~100-200MB |
| Real-time | Deterministic | GC Pauses |
| Hardware Support | Extensive | Limited |

## 🔧 Configuration

### TOML Configuration Example

```toml
[system]
max_nodes = 100
message_buffer_size = 1000
heartbeat_interval_ms = 1000
node_timeout_ms = 5000
log_level = "info"

[nodes.sensor_node]
enabled = true
namespace = "/sensors"
rate_hz = 50.0

[nodes.sensor_node.parameters]
sensor_type = "lidar"
port = "/dev/ttyUSB0"
```

### JSON Configuration Example

```json
{
  "system": {
    "max_nodes": 100,
    "message_buffer_size": 1000,
    "heartbeat_interval_ms": 1000,
    "node_timeout_ms": 5000,
    "log_level": "info"
  },
  "nodes": {
    "sensor_node": {
      "enabled": true,
      "namespace": "/sensors",
      "rate_hz": 50.0,
      "parameters": {
        "sensor_type": "lidar",
        "port": "/dev/ttyUSB0"
      }
    }
  }
}
```

## 🎯 Use Cases

### Highly Suitable

- **Service Robots**: Home and commercial service robots
- **Educational Robots**: Teaching and research platforms
- **Prototype Development**: Rapid prototype validation
- **Edge Computing**: Resource-constrained edge devices
- **Real-time Control**: Applications requiring deterministic latency

### Partially Suitable

- **Industrial Robots**: Requires additional safety certification
- **Autonomous Driving**: Requires more sensor fusion libraries
- **Medical Robots**: Requires medical-grade certification

## 🔌 Hardware Support

### Supported Platforms

- **High-end Embedded**: Jetson Nano/Xavier, Raspberry Pi 4/5
- **Industrial PCs**: Intel NUC, various x86 industrial computers
- **Microcontrollers**: ESP32, STM32, ARM Cortex-M
- **Development Boards**: Raspberry Pi Zero, BeagleBone

### Hardware Abstraction Layer

```rust
use robot_framework_rust::hal::*;

// GPIO control
let mut led = Gpio::new(18)?;
led.set_high()?;

// Serial communication
let mut uart = Uart::new("/dev/ttyUSB0", 115200)?;
uart.write(b"Hello Robot!")?;

// I2C sensor
let mut sensor = I2cDevice::new(0x48)?;
let data = sensor.read_register(0x00)?;
```

## 🧪 Testing

```bash
# Run all tests
cargo test

# Run benchmarks
cargo bench

# Run integration tests
cargo test --test integration

# Test coverage
cargo tarpaulin --out Html
```

## 📦 Deployment

### Cross Compilation

```bash
# ARM64 Linux
cargo build --target aarch64-unknown-linux-gnu --release

# ARM Linux (Raspberry Pi)
cargo build --target armv7-unknown-linux-gnueabihf --release

# Windows ARM64
cargo build --target aarch64-pc-windows-msvc --release
```

### Containerized Deployment

```dockerfile
FROM rust:1.70 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates
COPY --from=builder /app/target/release/robot-framework /usr/local/bin/
CMD ["robot-framework"]
```

### Embedded Deployment

```bash
# ESP32
cargo build --target xtensa-esp32-espidf --release

# STM32
cargo build --target thumbv7em-none-eabihf --release
```

## 🤝 Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for details.

### Development Environment Setup

```bash
# Install development dependencies
rustup component add clippy rustfmt
cargo install cargo-watch cargo-tarpaulin

# Code formatting
cargo fmt

# Code linting
cargo clippy

# Watch for file changes and auto-test
cargo watch -x test
```

## 📄 License

This project is licensed under either MIT or Apache-2.0 dual license. See [LICENSE-MIT](LICENSE-MIT) and [LICENSE-APACHE](LICENSE-APACHE) for details.

## 🔗 Related Links

- [Documentation](https://docs.rs/robot-framework-rust)
- [Examples](./examples/)
- [Changelog](CHANGELOG.md)
- [Roadmap](ROADMAP.md)

## 📞 Support

- 🐛 [Report Issues](https://github.com/your-org/robot-framework-rust/issues)
- 💬 [Discussions](https://github.com/your-org/robot-framework-rust/discussions)
- 📧 [Email Support](mailto:support@your-org.com)

---

**Robot Framework Rust** - Built for next-generation robotics applications 🤖✨