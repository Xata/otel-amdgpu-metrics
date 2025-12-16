use crate::AmdGpu;
use opentelemetry::KeyValue;
use opentelemetry::metrics::Meter;

/// Registers GPU metrics with the provided OpenTelemetry meter
pub fn register_gpu_metrics(meter: &Meter, gpus: &[AmdGpu]) {
    let gpus_util = gpus.to_vec();
    let _utilization = meter
        .u64_observable_gauge("hw.gpu.utilization")
        .with_description("GPU utilization percentage")
        .with_unit("%")
        .with_callback(move |observer| {
            for gpu in &gpus_util {
                if let Ok(util) = gpu.read_utilization() {
                    observer.observe(util, &[KeyValue::new("hw.id", gpu.card_id.clone())]);
                }
            }
        })
        .build();

    let gpus_vram = gpus.to_vec();
    let _vram_used = meter
        .u64_observable_gauge("hw.gpu.memory.used")
        .with_description("GPU memory used")
        .with_unit("By")
        .with_callback(move |observer| {
            for gpu in &gpus_vram {
                if let Ok(used) = gpu.read_vram_used() {
                    observer.observe(used, &[KeyValue::new("hw.id", gpu.card_id.clone())]);
                }
            }
        })
        .build();

    let gpus_temp = gpus.to_vec();
    let _temperature = meter
        .u64_observable_gauge("hw.gpu.temperature")
        .with_description("GPU temperature")
        .with_unit("Cel")
        .with_callback(move |observer| {
            for gpu in &gpus_temp {
                if let Ok(temp) = gpu.read_temperature() {
                    observer.observe(temp / 1000, &[KeyValue::new("hw.id", gpu.card_id.clone())]);
                }
            }
        })
        .build();
}
