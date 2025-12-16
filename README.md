# otel-amdgpu-metrics

A Rust library that collects AMD GPU metrics and exports them through OpenTelemetry.

## Introduction

This project provides OpenTelemetry instrumentation for AMD GPUs on Linux. It reads metrics directly from the sysfs filesystem (`/sys/class/drm/`) so there's no need to install ROCm or any other external dependencies. If your system has the `amdgpu` kernel driver loaded, this should work.

I built this because GPU metrics are often left out of observability setups or require vendor-specific tooling. With this library you can get GPU telemetry into whatever backend you're already using with OpenTelemetry.

This is not an official OpenTelemetry project. Just a community contribution.

## Metrics

The following metrics are collected for each detected AMD GPU:

| Metric | Description | Unit |
|--------|-------------|------|
| `hw.gpu.utilization` | GPU core utilization | % |
| `hw.gpu.memory.used` | VRAM currently in use | bytes |
| `hw.gpu.temperature` | GPU temperature | Celsius |

Each metric includes an `hw.id` attribute with the card identifier (e.g., `card0`, `card1`).

## Requirements

- Linux with the `amdgpu` kernel driver
- Rust 1.85 or later (uses 2024 edition)

Tested on EndeavourOS with a Radeon RX 7900 GRE (discrete) and Ryzen 7 9700X integrated graphics.

## Usage

Add to your `Cargo.toml`:
```toml
[dependencies]
otel-amdgpu-metrics = "0.1"
opentelemetry = "0.31"
```

Then initialize in your code:
```rust
use opentelemetry::global;
use otel_amdgpu_metrics::init;

let meter = global::meter("my-app");
let gpus = init(&meter).expect("Failed to detect AMD GPUs");

println!("Monitoring {} GPU(s)", gpus.len());
```

The library will automatically detect all AMD GPUs using the `amdgpu` driver and register metrics with your OpenTelemetry meter provider.

## Running the Example
```bash
cargo run --example basic
```

## License

Apache-2.0