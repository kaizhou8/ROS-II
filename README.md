# Robot Framework Rust / Rust 机器人框架

**English:**  
A high-performance, memory-safe robotics framework built with Rust, providing real-time capabilities, cross-platform support, and full ROS compatibility.

**中文:**  
使用 Rust 构建的高性能、内存安全的机器人框架，提供实时能力、跨平台支持和完整的 ROS 兼容性。

## 🚀 Features / 功能特性

- **Zero-cost abstractions / 零成本抽象**: High performance through Rust's type system / 通过 Rust 类型系统实现高性能
- **Memory safety / 内存安全**: Safety guarantees without runtime overhead / 无运行时开销的安全保证
- **Real-time capabilities / 实时能力**: Deterministic performance without garbage collection / 无垃圾回收的确定性性能
- **Cross-platform / 跨平台**: From microcontrollers to servers / 从微控制器到服务器
- **Async-first / 异步优先**: Efficient concurrency based on tokio / 基于 tokio 的高效并发
- **ROS compatible / ROS 兼容**: Complete ROS-style communication patterns / 完整的 ROS 风格通信模式
  - **Messaging / 消息**: Publish/subscribe pattern / 发布/订阅模式
  - **Services / 服务**: Request/response communication / 请求/响应通信
  - **Actions / 动作**: Long-running task management / 长时间运行任务管理
  - **Transforms / 变换**: TF system for coordinate frame management / 坐标系管理的 TF 系统

## 📋 System Requirements / 系统要求

- Rust 1.70+ 
- Supported platforms:
  - Windows (x86_64, ARM64)
  - Linux (x86_64, ARM64, ARM)
  - macOS (x86_64, ARM64)
  - Embedded platforms (ARM Cortex-M, ESP32, RISC-V)

## 🛠️ Quick Start / 快速开始

### Installation / 安装

```bash
# Clone repository
git clone https://github.com/your-org/robot-framework-rust.git
cd robot-framework-rust

# Build project
cargo build --release

# Run example
cargo run --example simple_robot
```

### Basic Usage / 基本使用

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

## 🏗️ Architecture / 架构

### Core Components / 核心组件

- **Node System / 节点系统**: Modular node architecture based on traits / 基于特征的模块化节点架构
- **Message System / 消息系统**: High-performance publish-subscribe messaging / 高性能发布订阅消息
- **Time System / 时间系统**: High-precision timestamps and rate control / 高精度时间戳和速率控制
- **Configuration System / 配置系统**: Flexible configuration management and parameter server / 灵活的配置管理和参数服务器
- **Logging System / 日志系统**: Structured logging / 结构化日志

### Message Types / 消息类型

**English:**  
The framework provides common robot message types:

**中文:**  
框架提供常见的机器人消息类型：

- `RobotActionMessage`: Robot action commands / 机器人动作命令
- `SensorDataMessage`: Sensor data / 传感器数据
- `SystemStatusMessage`: System status information / 系统状态信息
- `NavigationGoalMessage`: Navigation goals / 导航目标

## 📊 Performance Characteristics / 性能特征

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