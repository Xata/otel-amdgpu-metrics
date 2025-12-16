use opentelemetry::global;
use opentelemetry_sdk::metrics::SdkMeterProvider;
use opentelemetry_stdout::MetricExporter;
use otel_amdgpu_metrics::init;
use std::thread;
use std::time::Duration;

fn main() {
    let exporter = MetricExporter::default();
    let provider = SdkMeterProvider::builder()
        .with_periodic_exporter(exporter)
        .build();

    global::set_meter_provider(provider.clone());

    let meter = global::meter("otel-amdgpu-metrics");

    match init(&meter) {
        Ok(gpus) => println!("Monitoring {} AMD GPU(s)", gpus.len()),
        Err(e) => {
            eprintln!("Failed to initialize: {}", e);
            return;
        }
    }

    println!("Collecting metrics for 10 seconds...\n");
    thread::sleep(Duration::from_secs(10));

    let _ = provider.shutdown();
}
