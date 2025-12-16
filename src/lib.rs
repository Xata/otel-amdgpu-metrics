use std::fs;
use std::path::{Path, PathBuf};
use std::io;

use opentelemetry::metrics::Meter;

mod metrics;
pub use metrics::register_gpu_metrics;

/// Represents a detected AMD GPU
#[derive(Debug, Clone)]
pub struct AmdGpu {
    /// Card identifier (e.g., "card0", "card1")
    pub card_id: String,
    /// Path to the card's sysfs directory
    pub sysfs_path: PathBuf,
}

/// Detects AMD GPUs using the amdgpu kernel driver
pub fn detect_gpus() -> Vec<AmdGpu> {
    detect_gpus_at_path(Path::new("/sys/class/drm"))
}

/// Error type for initialization failures
#[derive(Debug)]
pub enum InitError {
    NoGpusFound,
}

fn detect_gpus_at_path(sysfs_drm_path: &Path) -> Vec<AmdGpu> {
    let mut gpus = Vec::new();

    let entries = match fs::read_dir(sysfs_drm_path) {
        Ok(entries) => entries,
        Err(_) => return gpus,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();

        // Only look at card directories (skip renderD* nodes)
        if !name.starts_with("card") || name.contains('-') {
            continue;
        }

        // Check if it's using the amdgpu driver
        if is_amdgpu(&path) {
            gpus.push(AmdGpu {
                card_id: name,
                sysfs_path: path,
            });
        }
    }

    gpus
}

fn is_amdgpu(card_path: &Path) -> bool {
    let driver_link = card_path.join("device/driver");
    
    match fs::read_link(&driver_link) {
        Ok(target) => target
            .to_string_lossy()
            .ends_with("amdgpu"),
        Err(_) => false,
    }
}

impl std::fmt::Display for InitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InitError::NoGpusFound => write!(f, "No AMD GPUs found with amdgpu driver"),
        }
    }
}

impl std::error::Error for InitError {}

/// Initialize AMD GPU metrics collection
/// 
/// Detects AMD GPUs and registers metrics with the provided meter.
/// Returns the list of detected GPUs.
/// 
/// # Example
/// ```ignore
/// use opentelemetry::global;
/// use otel_amdgpu_metrics::init;
/// 
/// let meter = global::meter("my-app");
/// let gpus = init(&meter)?;
/// println!("Monitoring {} GPU(s)", gpus.len());
/// ```
pub fn init(meter: &Meter) -> Result<Vec<AmdGpu>, InitError> {
    let gpus = detect_gpus();
    
    if gpus.is_empty() {
        return Err(InitError::NoGpusFound);
    }
    
    register_gpu_metrics(meter, &gpus);
    Ok(gpus)
}

impl AmdGpu {
    /// Read GPU utilization as a percentage (0-100)
    pub fn read_utilization(&self) -> io::Result<u64> {
        let path = self.sysfs_path.join("device/gpu_busy_percent");
        let content = fs::read_to_string(&path)?;
        content
            .trim()
            .parse()
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    /// Read VRAM used in bytes
    pub fn read_vram_used(&self) -> io::Result<u64> {
        let path = self.sysfs_path.join("device/mem_info_vram_used");
        let content = fs::read_to_string(&path)?;
        content
            .trim()
            .parse()
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    /// Read total VRAM in bytes
    pub fn read_vram_total(&self) -> io::Result<u64> {
        let path = self.sysfs_path.join("device/mem_info_vram_total");
        let content = fs::read_to_string(&path)?;
        content
            .trim()
            .parse()
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

        /// Read GPU temperature in millidegrees Celsius
    pub fn read_temperature(&self) -> io::Result<u64> {
        // hwmon path can vary, need to find it
        let hwmon_path = self.find_hwmon()?;
        let path = hwmon_path.join("temp1_input");
        let content = fs::read_to_string(&path)?;
        content
            .trim()
            .parse()
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    /// Read power consumption in microwatts
    pub fn read_power(&self) -> io::Result<u64> {
        let hwmon_path = self.find_hwmon()?;
        let path = hwmon_path.join("power1_average");
        let content = fs::read_to_string(&path)?;
        content
            .trim()
            .parse()
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    /// Find the hwmon directory for this GPU
    fn find_hwmon(&self) -> io::Result<PathBuf> {
        let hwmon_dir = self.sysfs_path.join("device/hwmon");
        
        for entry in fs::read_dir(&hwmon_dir)? {
            let entry = entry?;
            if entry.file_name().to_string_lossy().starts_with("hwmon") {
                return Ok(entry.path());
            }
        }
        
        Err(io::Error::new(
            io::ErrorKind::NotFound,
            "hwmon directory not found",
        ))
    }
}